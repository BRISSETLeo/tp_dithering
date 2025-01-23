use image::{self, GrayImage};
use image::Pixel;
use crate::WHITE;
use crate::BLACK;

pub fn diffusion_erreur(img_rgb: &image::RgbImage, img_out: &mut image::RgbImage) -> image::RgbImage {
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