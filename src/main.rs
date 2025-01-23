use argh::FromArgs;
use image::{self, ImageError};
use image::Pixel;

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
    DiffusionErreur(OptsDiffusionErreur)
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

fn error_diffusion_dithering(img_gray: &image::GrayImage) -> image::GrayImage {
    let (width, height) = img_gray.dimensions();
    let mut img_out = image::ImageBuffer::new(width, height);
    let mut error = vec![vec![0.0; width as usize]; height as usize];
    
    for y in 0..height {
        for x in 0..width {
            let pixel = img_gray.get_pixel(x, y).0[0] as f64;
            let new_pixel = if pixel + error[y as usize][x as usize] > 127.0 { 255 } else { 0 };
            let quant_error = pixel - new_pixel as f64;
            img_out.put_pixel(x, y, image::Luma([new_pixel as u8]));
            
            if x + 1 < width {
                error[y as usize][x as usize + 1] += quant_error * 7.0 / 16.0;
            }
            if y + 1 < height {
                error[y as usize + 1][x as usize] += quant_error * 5.0 / 16.0;
                if x > 0 {
                    error[y as usize + 1][x as usize - 1] += quant_error * 3.0 / 16.0;
                }
                if x + 1 < width {
                    error[y as usize + 1][x as usize + 1] += quant_error * 1.0 / 16.0;
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
    
    let img_rgb = img.to_rgb8();

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
            let palette = vec![BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
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
            // Convertir l'image en niveaux de gris
            let img_gray = img.to_luma8();
            
            // Appliquer l'algorithme de diffusion d'erreur
            let img_dithered = error_diffusion_dithering(&img_gray);
            
            // Copier les pixels dans l'image de sortie (en RGB)
            for (x, y, pixel) in img_dithered.enumerate_pixels() {
                let color = if pixel[0] == 255 { WHITE } else { BLACK };
                img_out.put_pixel(x, y, color);
            }
        }
        
    }
    
    

    img_out.save(path_out)?;

    Ok(())
}

fn string_to_color(couleur: &str) -> image::Rgb<u8> {
    match couleur {
        "noir" => BLACK,
        "blanc" => WHITE,
        "gris" => GREY,
        "rouge" => RED,
        "vert" => GREEN,
        "bleu" => BLUE,
        "jaune" => YELLOW,
        "cyan" => CYAN,
        "magenta" => MAGENTA,
        _ => BLACK
    }
}

fn color_distance(color1: image::Rgb<u8>, color2: image::Rgb<u8>) -> f64 {
    let r_diff = (color1[0] as i32 - color2[0] as i32).pow(2);
    let g_diff = (color1[1] as i32 - color2[1] as i32).pow(2);
    let b_diff = (color1[2] as i32 - color2[2] as i32).pow(2);
    ((r_diff + g_diff + b_diff) as f64).sqrt()
}
