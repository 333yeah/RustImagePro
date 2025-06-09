use image::{DynamicImage, Rgb, ImageBuffer};

pub fn sharpen_image(img: &DynamicImage, amount: f32) -> DynamicImage {
    let img = img.to_rgb8();
    let (width, height) = img.dimensions();
    let mut new_img = ImageBuffer::new(width, height);

    // Laplacian kernel for sharpening
    let kernel = [
        [0.0, -1.0, 0.0],
        [-1.0, 5.0, -1.0],
        [0.0, -1.0, 0.0],
    ];

    // Apply sharpening
    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut weight_sum = 0.0;

            // Apply convolution kernel
            for ky in -1..=1 {
                for kx in -1..=1 {
                    let nx = x as i32 + kx;
                    let ny = y as i32 + ky;
                    
                    // Boundary handling: mirror padding
                    let (nx, ny) = if nx < 0 {
                        (-nx, ny)
                    } else if nx >= width as i32 {
                        (2 * width as i32 - nx - 1, ny)
                    } else {
                        (nx, ny)
                    };
                    
                    let (nx, ny) = if ny < 0 {
                        (nx, -ny)
                    } else if ny >= height as i32 {
                        (nx, 2 * height as i32 - ny - 1)
                    } else {
                        (nx, ny)
                    };

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let pixel = img.get_pixel(nx as u32, ny as u32);
                        let weight = kernel[(ky + 1) as usize][(kx + 1) as usize];
                        
                        let edge_factor = if x < 2 || x >= width - 2 || y < 2 || y >= height - 2 {
                            0.5
                        } else {
                            1.0
                        };
                        
                        let adjusted_weight = weight * edge_factor;
                        
                        sum_r += pixel[0] as f32 * adjusted_weight;
                        sum_g += pixel[1] as f32 * adjusted_weight;
                        sum_b += pixel[2] as f32 * adjusted_weight;
                        weight_sum += adjusted_weight;
                    }
                }
            }

            // 归一化并应用锐化强度
            let scale = 1.0 / weight_sum;
            let r = ((sum_r * scale * amount + img.get_pixel(x, y)[0] as f32 * (1.0 - amount))
                .clamp(0.0, 255.0)) as u8;
            let g = ((sum_g * scale * amount + img.get_pixel(x, y)[1] as f32 * (1.0 - amount))
                .clamp(0.0, 255.0)) as u8;
            let b = ((sum_b * scale * amount + img.get_pixel(x, y)[2] as f32 * (1.0 - amount))
                .clamp(0.0, 255.0)) as u8;

            new_img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    DynamicImage::ImageRgb8(new_img)
} 