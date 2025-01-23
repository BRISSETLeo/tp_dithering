use image::{Rgb};

pub const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
pub const GREY: Rgb<u8> = Rgb([127, 127, 127]);
pub const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
pub const BLUE: Rgb<u8> = Rgb([0, 0, 255]);
pub const RED: Rgb<u8> = Rgb([255, 0, 0]);
pub const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
pub const YELLOW: Rgb<u8> = Rgb([255, 255, 0]);
pub const MAGENTA: Rgb<u8> = Rgb([255, 0, 255]);
pub const CYAN: Rgb<u8> = Rgb([0, 255, 255]);

pub fn string_to_color(couleur: &str) -> Rgb<u8> {
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
        _ => BLACK,
    }
}

pub fn color_distance(color1: Rgb<u8>, color2: Rgb<u8>) -> f64 {
    let r_diff = (color1[0] as i32 - color2[0] as i32).pow(2);
    let g_diff = (color1[1] as i32 - color2[1] as i32).pow(2);
    let b_diff = (color1[2] as i32 - color2[2] as i32).pow(2);
    ((r_diff + g_diff + b_diff) as f64).sqrt()
}
