use argh::FromArgs;
use image::{self, ImageError};
use image::Pixel;


mod utils;
mod bayer; 
mod error_diffusion;

use utils::*;
use bayer::*;
use error_diffusion::*;
use crate::image::Rgb;
use crate::image::Rgba;

#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette rÃ©duite de couleurs.
struct DitherArgs {

    /// le fichier dâentrÃ©e
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,

    /// le mode dâopÃ©ration
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Tramage(OptsTramage),
    TramageBayer(OptsTramageBayer),
    DiffusionErreur(OptsDiffusionErreur),
    DiffusionErreurPalette(OptsDiffusionErreurPalette),
    DiffusionErreurFloyd(OptsDiffusionErreurPaletteFloyd),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de lâimage par seuillage monochrome.
struct OptsSeuil {
    
    /// la couleur pour les pixels en dessous du seuil (optionnel, par défaut noir)
    #[argh(option, default = "\"noir\".to_string()")]
    couleur_bas: String,

    /// la couleur pour les pixels au-dessus du seuil (optionnel, par défaut blanc)
    #[argh(option, default = "\"blanc\".to_string()")]
    couleur_haut: String,
}


#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de lâimage avec une palette contenant un nombre limitÃ© de couleurs
struct OptsPalette {

    /// le nombre de couleurs Ã  utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramage")]
/// Rendu de lâimage par tramage
struct OptsTramage {
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramage-bayer")]
/// Rendu de lâimage par tramage avec matrice de Bayer
struct OptsTramageBayer {
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion-erreur")]
/// Rendu de lâimage par diffusion dâerreur
struct OptsDiffusionErreur {
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion-erreur-palette")]
/// Rendu de lâimage par diffusion dâerreur avec palette
struct OptsDiffusionErreurPalette {
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion-erreur-floyd")]
/// Rendu de lâimage par diffusion dâerreur de Floyd-Steinberg
struct OptsDiffusionErreurPaletteFloyd {
}

 
const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const GREY: image::Rgb<u8> = image::Rgb([127, 127, 127]);
const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);

fn generate_bayer_matrix(order: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; order]; order];
    matrix[0][0] = 0.0;

    for n in 1..=(order as f64).log2() as usize {
        let current_size = 2_usize.pow(n as u32);
        let half_size = current_size / 2;

        for i in 0..half_size {
            for j in 0..half_size {
                matrix[i][j] *= 4.0;
                matrix[i][j + half_size] = matrix[i][j] + 2.0;
                matrix[i + half_size][j] = matrix[i][j] + 3.0;
                matrix[i + half_size][j + half_size] = matrix[i][j] + 1.0;
            }
        }
    }

    // Normalisation
    matrix.iter_mut()
        .flat_map(|row| row.iter_mut())
        .for_each(|val| *val /= (order * order) as f64);

    matrix
}

fn apply_bayer_dithering(img_rgb: &image::RgbImage) -> image::RgbImage {
    let (width, height) = img_rgb.dimensions();
    let mut img_out = image::ImageBuffer::new(width, height);
    
    // Générer une matrice de Bayer 8x8
    let bayer_matrix = generate_bayer_matrix(8);
    
    for (x, y, pixel) in img_rgb.enumerate_pixels() {
        let luma = pixel.to_luma().0[0] as f64 / 255.0;
        let seuil = bayer_matrix[y as usize % 8][x as usize % 8];
        
        let output_pixel = if luma > seuil { WHITE } else { BLACK };
        img_out.put_pixel(x, y, output_pixel);
    }
    
    img_out
}

fn diffusion_erreur(img_rgb: &image::RgbImage, img_out: &mut image::RgbImage) -> image::RgbImage {
    let mut img_rgb = img_rgb.clone();
    let width = img_rgb.width();
    let height = img_rgb.height();

    for y in 0..height {
        for x in 0..width {
            let pixel = img_rgb.get_pixel(x, y);
            let luma = pixel.to_luma().0[0] as f64 / 255.0;
            let new_pixel = if luma > 0.5 { WHITE } else { BLACK };
            img_out.put_pixel(x, y, new_pixel);

            let error = luma - if luma > 0.5 { 1.0 } else { 0.0 };

            if x + 1 < width {
                let next_pixel = img_rgb.get_pixel(x + 1, y);
                let next_luma = next_pixel.to_luma().0[0] as f64 / 255.0;
                let new_luma = (next_luma + error * 0.5).clamp(0.0, 1.0);
                let new_color = image::Luma([(new_luma * 255.0) as u8]);
                img_rgb.put_pixel(x + 1, y, image::Rgb([new_color[0], new_color[0], new_color[0]]));
            }

            if y + 1 < height {
                let next_pixel = img_rgb.get_pixel(x, y + 1);
                let next_luma = next_pixel.to_luma().0[0] as f64 / 255.0;
                let new_luma = (next_luma + error * 0.5).clamp(0.0, 1.0);
                let new_color = image::Luma([(new_luma * 255.0) as u8]);
                img_rgb.put_pixel(x, y + 1, image::Rgb([new_color[0], new_color[0], new_color[0]]));
            }
        }
    }

    return img_rgb
}

fn diffusion_erreur_palette(
    img_rgb: &image::RgbImage,
    palette: &[image::Rgb<u8>],
    error_matrix: &[Vec<f32>],
    x_origin: usize,
    y_origin: usize,
) -> image::RgbImage {
    let (width, height) = img_rgb.dimensions();
    let mut img_out = img_rgb.clone();

    for y in 0..height {
        for x in 0..width {
            let original_pixel = img_out.get_pixel(x, y);
            let closest_color = palette
                .iter()
                .min_by(|&&a, &&b| {
                    color_distance(*original_pixel, a).total_cmp(&color_distance(*original_pixel, b))
                })
                .unwrap();

            let error = [
                original_pixel[0] as f32 - closest_color[0] as f32,
                original_pixel[1] as f32 - closest_color[1] as f32,
                original_pixel[2] as f32 - closest_color[2] as f32,
            ];

            img_out.put_pixel(x, y, *closest_color);

            for y_error in 0..error_matrix.len() {
                for x_error in 0..error_matrix[y_error].len() {
                    let error_value = error_matrix[y_error][x_error];
                    let target_x = x as i32 + (x_error as i32 - x_origin as i32);
                    let target_y = y as i32 + (y_error as i32 - y_origin as i32);

                    if target_x >= 0 && target_x < width as i32 && target_y >= 0 && target_y < height as i32 {
                        let mut neighbor_pixel = img_out.get_pixel_mut(target_x as u32, target_y as u32);
                        for i in 0..3 {
                            let adjusted_value = (neighbor_pixel[i] as f32 + error[i] * error_value).clamp(0.0, 255.0);
                            neighbor_pixel[i] = adjusted_value as u8;
                        }
                    }
                }
            }
        }
    }

    img_out
}

fn diffusion_erreur_floyd_steinberg(
    img_rgb: &image::RgbImage,
    palette: &[image::Rgb<u8>],
) -> image::RgbImage {
    let (width, height) = img_rgb.dimensions();
    let mut img_out = img_rgb.clone();

    // Matrice de diffusion de Floyd-Steinberg
    let error_matrix = vec![
        vec![0.0, 0.0, 7.0 / 16.0],
        vec![3.0 / 16.0, 5.0 / 16.0, 1.0 / 16.0],
    ];

    let matrix_width = error_matrix[0].len();
    let matrix_height = error_matrix.len();
    let x_origin = 1; // Position du pixel courant dans la matrice
    let y_origin = 0;

    for y in 0..height {
        for x in 0..width {
            let original_pixel = img_out.get_pixel(x, y);
            let closest_color = palette
                .iter()
                .min_by(|&&a, &&b| {
                    color_distance(*original_pixel, a).total_cmp(&color_distance(*original_pixel, b))
                })
                .unwrap();

            let error = [
                original_pixel[0] as f32 - closest_color[0] as f32,
                original_pixel[1] as f32 - closest_color[1] as f32,
                original_pixel[2] as f32 - closest_color[2] as f32,
            ];

            img_out.put_pixel(x, y, *closest_color);

            // Diffusion de l'erreur
            for y_error in 0..matrix_height {
                for x_error in 0..matrix_width {
                    let error_value = error_matrix[y_error][x_error];
                    let target_x = x as i32 + (x_error as i32 - x_origin as i32);
                    let target_y = y as i32 + (y_error as i32 - y_origin as i32);

                    if target_x >= 0 && target_x < width as i32 && target_y >= 0 && target_y < height as i32 {
                        let mut neighbor_pixel = img_out.get_pixel_mut(target_x as u32, target_y as u32);
                        for i in 0..3 {
                            let adjusted_value = (neighbor_pixel[i] as f32 + error[i] * error_value).clamp(0.0, 255.0);
                            neighbor_pixel[i] = adjusted_value as u8;
                        }
                    }
                }
            }
        }
    }

    img_out
}



fn main() -> Result<(), ImageError>{
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out = args.output.unwrap_or("./image/out.png".to_string());

    let img = image::open(path_in)?;

    println!("Image ouverte: {}x{}", img.width(), img.height());
    
    let mut img_rgb = img.to_rgb8();

    let mut img_out = image::ImageBuffer::new(img_rgb.width(), img_rgb.height());

    let pixel = img_rgb.get_pixel(32, 52);
    println!("Couleur du pixel (32, 52): {:?}", pixel);

    println!("Mode: {:?}", args.mode);

    match args.mode {
        Mode::Seuil(opts) => {
            let couleur_bas = string_to_color(&opts.couleur_bas);
            let couleur_haut = string_to_color(&opts.couleur_haut);
            for (x, y, pixel) in img_rgb.enumerate_pixels() {
                // if (x + y) % 2 == 0 {
                //     img_out.put_pixel(x, y, WHITE);
                // } else {
                //     img_out.put_pixel(x, y, *pixel);
                // }
                if pixel.to_luma().0[0] > 127{
                    img_out.put_pixel(x, y, couleur_haut);
                } else {
                    img_out.put_pixel(x, y, couleur_bas);
                }
            }
        },
        Mode::Palette(opts) => {
            let palette = vec![BLACK, WHITE, RED, BLUE, GREEN];
            let nb_couleur = opts.n_couleurs;
            if nb_couleur < 2 {
                return Err(image::ImageError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Le nombre de couleurs doit être supérieur ou égal à 2.",
                )));
            } else if nb_couleur > palette.len() {
                return Err(image::ImageError::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Le nombre de couleurs doit être inférieur ou égal à 8.",
                )));
            }
            let limited_palette: Vec<image::Rgb<u8>> = palette.into_iter().take(nb_couleur).collect();
            for (x, y, pixel) in img_rgb.enumerate_pixels() {
                let closest_color = limited_palette.iter()
                    .min_by(|&&a, &&b| {
                        color_distance(*pixel, a).total_cmp(&color_distance(*pixel, b))
                    })
                    .unwrap();
                img_out.put_pixel(x, y, *closest_color);
            }
        },
        Mode::Tramage(_opts) => {
            for (x, y, pixel) in img_rgb.enumerate_pixels() {
                let threshold: u8 = rand::random::<u8>();
                let pixel_luma = pixel.to_luma().0[0];
                if pixel_luma > threshold {
                    img_out.put_pixel(x, y, WHITE);
                } else {
                    img_out.put_pixel(x, y, BLACK);
                }
            }
        },
        Mode::TramageBayer(_) => {
            img_out = apply_bayer_dithering(&img_rgb);
        },
        Mode::DiffusionErreur(_) => {
            diffusion_erreur(&img_rgb, &mut img_out);
        },
        Mode::DiffusionErreurPalette(opts) => {
            let palette = vec![BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
        
            let error_matrix = vec![
                vec![0.0, 0.5],
                vec![0.5, 0.0],
            ];
        
            img_out = diffusion_erreur_palette(&img_rgb, &palette, &error_matrix, 0, 0);
        },
        Mode::DiffusionErreurFloyd(_) => {
            let palette = vec![BLACK, WHITE]; // Palette monochrome par défaut
            img_out = diffusion_erreur_floyd_steinberg(&img_rgb, &palette);
        }
        
        
    
        
    }

    img_out.save(path_out)?;

    Ok(())
}