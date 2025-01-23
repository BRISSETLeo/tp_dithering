use image::{self, GrayImage};

pub fn error_diffusion_dithering(img_gray: &GrayImage) -> GrayImage {
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