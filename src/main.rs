use argh::FromArgs;
use image::{self, ImageError};
use image::Rgb;


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
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de lâimage par seuillage monochrome.
struct OptsSeuil {}


#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de lâimage avec une palette contenant un nombre limitÃ© de couleurs
struct OptsPalette {

    /// le nombre de couleurs Ã  utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
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


fn distance_couleur(c1: Rgb<u8>, c2: Rgb<u8>) -> f64 {
    let r_diff = c1[0] as f64 - c2[0] as f64;
    let g_diff = c1[1] as f64 - c2[1] as f64;
    let b_diff = c1[2] as f64 - c2[2] as f64;

    (r_diff.powi(2) + g_diff.powi(2) + b_diff.powi(2)).sqrt()
}

fn appliquer_palette(
    img: &image::RgbImage,
    palette: &[Rgb<u8>],
) -> image::RgbImage {
    let mut img_out = image::ImageBuffer::new(img.width(), img.height());

    for (x, y, pixel) in img.enumerate_pixels() {
        let mut couleur_proche = palette[0];
        let mut distance_min = f64::MAX;

        // Trouver la couleur la plus proche dans la palette
        for &couleur in palette {
            let distance = distance_couleur(*pixel, couleur);
            if distance < distance_min {
                distance_min = distance;
                couleur_proche = couleur;
            }
        }

        img_out.put_pixel(x, y, couleur_proche);
    }

    img_out
}

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out = args.output.unwrap_or("./image/out.png".to_string());

    let img = image::open(path_in)?.to_rgb8();

    match args.mode {
        Mode::Seuil(_) => {
            println!("Mode seuil non implémenté");
        }
        Mode::Palette(opts) => {
            let palette = vec![BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
            let palette_reduite = &palette[..opts.n_couleurs.min(palette.len())];
            let img_out = appliquer_palette(&img, palette_reduite);
            img_out.save(path_out)?;
        }
    }

    Ok(())
}

