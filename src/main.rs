use argh::FromArgs;
use image::{self, ImageError};
use image::Pixel;


mod utils;
mod bayer; 
mod error_diffusion;

use utils::*;
use bayer::*;
use error_diffusion::*;

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