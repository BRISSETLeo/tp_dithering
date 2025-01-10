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
    Tramage(OptsTramage)
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
 
const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const GREY: image::Rgb<u8> = image::Rgb([127, 127, 127]);
const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);

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
        Mode::Tramage(opts) => {
            let palette = vec![BLACK, WHITE];
            for (x, y, pixel) in img_rgb.enumerate_pixels() {
                let threshold: u8 = rand::random::<u8>();
                let pixel_luma = pixel.to_luma().0[0];
                if pixel_luma > threshold {
                    img_out.put_pixel(x, y, WHITE);
                } else {
                    img_out.put_pixel(x, y, BLACK);
                }
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
