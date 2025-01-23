use image::{Rgb, RgbImage};
use image::Pixel;

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const BLACK: Rgb<u8> = Rgb([0, 0, 0]);

pub fn generate_bayer_matrix(order: usize) -> Vec<Vec<f64>> {
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

    matrix.iter_mut()
        .flat_map(|row| row.iter_mut())
        .for_each(|val| *val /= (order * order) as f64);

    matrix
}

pub fn apply_bayer_dithering(img_rgb: &RgbImage) -> RgbImage {
    let (width, height) = img_rgb.dimensions();
    let mut img_out = RgbImage::new(width, height);

    let bayer_matrix = generate_bayer_matrix(8);

    for (x, y, pixel) in img_rgb.enumerate_pixels() {
        let luma = pixel.to_luma().0[0] as f64 / 255.0;
        let seuil = bayer_matrix[y as usize % 8][x as usize % 8];

        let output_pixel = if luma > seuil { WHITE } else { BLACK };
        img_out.put_pixel(x, y, output_pixel);
    }

    img_out
}
