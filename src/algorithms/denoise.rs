use image::{DynamicImage, Rgb, ImageBuffer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DenoiseType {
    MeanFilter,
    GaussianFilter,
    MedianFilter,
    BilateralFilter,
    NonLocalMeans,
    TotalVariation,
}

pub fn denoise_image(
    img: &DynamicImage,
    denoise_type: DenoiseType,
    kernel_size: usize,
    tv_lambda: f32,
    tv_iterations: usize,
) -> DynamicImage {
    let img = img.to_rgb8();
    let (width, height) = (img.width(), img.height());
    let mut new_img = ImageBuffer::new(width, height);
    let radius = kernel_size / 2;

    match denoise_type {
        DenoiseType::MeanFilter => mean_filter(&img, &mut new_img, width, height, radius),
        DenoiseType::GaussianFilter => gaussian_filter(&img, &mut new_img, width, height, radius),
        DenoiseType::MedianFilter => median_filter(&img, &mut new_img, width, height, radius),
        DenoiseType::BilateralFilter => bilateral_filter(&img, &mut new_img, width, height, radius),
        DenoiseType::NonLocalMeans => non_local_means(&img, &mut new_img, width, height),
        DenoiseType::TotalVariation => total_variation(&img, &mut new_img, width, height, tv_lambda, tv_iterations),
    }

    DynamicImage::ImageRgb8(new_img)
}

fn mean_filter(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    new_img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    radius: usize,
) {
    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            let mut count = 0;
            
            for dy in 0..=radius*2 {
                for dx in 0..=radius*2 {
                    let nx = x as i32 + dx as i32 - radius as i32;
                    let ny = y as i32 + dy as i32 - radius as i32;
                    
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let pixel = img.get_pixel(nx as u32, ny as u32);
                        sum_r += pixel[0] as u32;
                        sum_g += pixel[1] as u32;
                        sum_b += pixel[2] as u32;
                        count += 1;
                    }
                }
            }
            
            let avg_r = (sum_r / count) as u8;
            let avg_g = (sum_g / count) as u8;
            let avg_b = (sum_b / count) as u8;
            new_img.put_pixel(x, y, Rgb([avg_r, avg_g, avg_b]));
        }
    }
}

fn gaussian_filter(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    new_img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    radius: usize,
) {
    let sigma = radius as f32 / 2.0;
    let mut kernel = vec![vec![0.0; radius*2+1]; radius*2+1];
    let mut sum = 0.0;

    // 生成高斯核
    for y in 0..=radius*2 {
        for x in 0..=radius*2 {
            let dx = x as f32 - radius as f32;
            let dy = y as f32 - radius as f32;
            let value = (-(dx*dx + dy*dy) / (2.0 * sigma * sigma)).exp();
            kernel[y][x] = value;
            sum += value;
        }
    }

    // 归一化
    for y in 0..=radius*2 {
        for x in 0..=radius*2 {
            kernel[y][x] /= sum;
        }
    }

    // 应用高斯滤波
    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            
            for dy in 0..=radius*2 {
                for dx in 0..=radius*2 {
                    let nx = x as i32 + dx as i32 - radius as i32;
                    let ny = y as i32 + dy as i32 - radius as i32;
                    
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let pixel = img.get_pixel(nx as u32, ny as u32);
                        let weight = kernel[dy][dx];
                        sum_r += pixel[0] as f32 * weight;
                        sum_g += pixel[1] as f32 * weight;
                        sum_b += pixel[2] as f32 * weight;
                    }
                }
            }
            
            let r = sum_r.clamp(0.0, 255.0) as u8;
            let g = sum_g.clamp(0.0, 255.0) as u8;
            let b = sum_b.clamp(0.0, 255.0) as u8;
            new_img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }
}

fn median_filter(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    new_img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    radius: usize,
) {
    for y in 0..height {
        for x in 0..width {
            let mut r_values = Vec::new();
            let mut g_values = Vec::new();
            let mut b_values = Vec::new();
            
            for dy in 0..=radius*2 {
                for dx in 0..=radius*2 {
                    let nx = x as i32 + dx as i32 - radius as i32;
                    let ny = y as i32 + dy as i32 - radius as i32;
                    
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let pixel = img.get_pixel(nx as u32, ny as u32);
                        r_values.push(pixel[0]);
                        g_values.push(pixel[1]);
                        b_values.push(pixel[2]);
                    }
                }
            }
            
            r_values.sort();
            g_values.sort();
            b_values.sort();
            
            let median_index = r_values.len() / 2;
            let r = r_values[median_index];
            let g = g_values[median_index];
            let b = b_values[median_index];
            
            new_img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }
}


fn bilateral_filter(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    new_img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    radius: usize,
) {
    let sigma_d = radius as f32; // Spatial domain standard deviation
    let sigma_r = 30.0; // Range domain standard deviation

    for y in 0..height {
        for x in 0..width {
            let center_pixel = img.get_pixel(x, y);
            let mut sums = [0.0f32; 3];
            let mut weight_sum = 0.0;

            for dy in 0..=radius*2 {
                for dx in 0..=radius*2 {
                    let nx = x as i32 + dx as i32 - radius as i32;
                    let ny = y as i32 + dy as i32 - radius as i32;
                    
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbor_pixel = img.get_pixel(nx as u32, ny as u32);
                        
                        // Calculate spatial weight
                        let x_diff = (dx as f32 - radius as f32).powf(2.0);
                        let y_diff = (dy as f32 - radius as f32).powf(2.0);
                        let spatial_weight = (-((x_diff + y_diff) / (2.0 * sigma_d * sigma_d))).exp();
                        
                        // Calculate range weight
                        let mut intensity_diff = 0.0;
                        for c in 0..3 {
                            intensity_diff += (center_pixel[c] as f32 - neighbor_pixel[c] as f32).powf(2.0);
                        }
                        intensity_diff /= 3.0;
                        let range_weight = (-intensity_diff / (2.0 * sigma_r * sigma_r)).exp();
                        
                        let weight = spatial_weight * range_weight;
                        for c in 0..3 {
                            sums[c] += neighbor_pixel[c] as f32 * weight;
                        }
                        weight_sum += weight;
                    }
                }
            }
            
            let pixel = [
                (sums[0] / weight_sum) as u8,
                (sums[1] / weight_sum) as u8,
                (sums[2] / weight_sum) as u8,
            ];
            new_img.put_pixel(x, y, Rgb(pixel));
        }
    }
}

fn non_local_means(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    new_img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
) {
    let ds = 2; // Block size for calculating the weight
    let Ds = 5; // Search window size
    let h = 10.0; // Decay factor

    // Create padded image
    let offset = ds + Ds;
    let offset_u32 = offset as u32;
    let mut padded_img = ImageBuffer::new(width + 2 * offset_u32, height + 2 * offset_u32);

    // Use symmetric padding
    for y in 0..height + 2 * offset_u32 {
        for x in 0..width + 2 * offset_u32 {
            let src_x = if x < offset_u32 {
                offset_u32 - x - 1
            } else if x >= width + offset_u32 {
                2 * width + offset_u32 - x - 1
            } else {
                x - offset_u32
            };
            
            let src_y = if y < offset_u32 {
                offset_u32 - y - 1
            } else if y >= height + offset_u32 {
                (2 * height + offset_u32).checked_sub(y).map_or(0, |val| val - 1)
            } else {
                y - offset_u32
            };
            
            padded_img.put_pixel(x, y, *img.get_pixel(src_x.min(width-1), src_y.min(height-1)));
        }
    }

    let mut sum_image = vec![vec![0.0f32; 3]; (width * height) as usize];
    let mut sum_weight = vec![0.0; (width * height) as usize];
    let mut max_weight = vec![0.0; (width * height) as usize];

    // Iterate over the search window
    for r in -Ds..=Ds {
        for s in -Ds..=Ds {
            if r == 0 && s == 0 {
                continue;
            }

            // Calculate the patch distance integral image
            let mut diff = vec![0.0; (width + 2 * offset_u32) as usize * (height + 2 * offset_u32) as usize];
            
            for y in offset_u32..height + offset_u32 {
                for x in offset_u32..width + offset_u32 {
                    let base_y = y as i32;
                    let base_x = x as i32;
                    let offset_y = (base_y + r).max(0) as u32;
                    let offset_x = (base_x + s).max(0) as u32;
                    
                    if offset_y < height + 2 * offset_u32 && offset_x < width + 2 * offset_u32 {
                        let p1 = padded_img.get_pixel(base_x as u32, base_y as u32);
                        let p2 = padded_img.get_pixel(offset_x, offset_y);
                        let mut d = 0.0;
                        for c in 0..3 {
                            d += (p1[c] as f32 - p2[c] as f32).powf(2.0);
                        }
                        d /= 3.0;
                        let idx = ((y - offset_u32) * (width + 2 * offset_u32) + (x - offset_u32)) as usize;
                        if idx < diff.len() {
                            diff[idx] = d;
                        }
                    }
                }
            }

            // Calculate the integral image
            let mut integral = vec![0.0; (width + 2 * offset_u32) as usize * (height + 2 * offset_u32) as usize];

            // Horizontal summation
            for y in 0..height + 2 * offset_u32 {
                let mut sum = 0.0;
                for x in 0..width + 2 * offset_u32 {
                    let idx = (y * (width + 2 * offset_u32) + x) as usize;
                    if idx < diff.len() {
                        sum += diff[idx];
                        integral[idx] = sum;
                    }
                }
            }

            // Vertical summation
            for x in 0..width + 2 * offset_u32 {
                let mut sum = 0.0;
                for y in 0..height + 2 * offset_u32 {
                    let idx = (y * (width + 2 * offset_u32) + x) as usize;
                    if idx < integral.len() {
                        sum += integral[idx];
                        integral[idx] = sum;
                    }
                }
            }

            // Compute pixel weights and update pixel values
            for y in 0..height {
                for x in 0..width {
                    let i = (y * width + x) as usize;
                    let window_size = (2 * ds + 1) as u32;
                    let top_right = ((y + window_size) * (width + 2 * offset_u32) + (x + window_size)) as usize;
                    let top_left = ((y + window_size) * (width + 2 * offset_u32) + x) as usize;
                    let bottom_right = ((y) * (width + 2 * offset_u32) + (x + window_size)) as usize;
                    let bottom_left = ((y) * (width + 2 * offset_u32) + x) as usize;

                    if top_right < integral.len() && top_left < integral.len() &&
                       bottom_right < integral.len() && bottom_left < integral.len() {
                        let distance = integral[top_right] + integral[bottom_left] 
                                       - integral[top_left] - integral[bottom_right];
                        
                        let distance = distance / ((window_size * window_size) as f32);
                        let weight = (-distance / (h * h)).exp();
                        
                        // Retrieve the offset pixel value
                        let offset_y = ((y + offset_u32) as i32 + r).max(0) as u32;
                        let offset_x = ((x + offset_u32) as i32 + s).max(0) as u32;
                        
                        if offset_y < height + 2 * offset_u32 && offset_x < width + 2 * offset_u32 {
                            let pixel = padded_img.get_pixel(offset_x, offset_y);
                            for c in 0..3 {
                                sum_image[i][c] += weight * pixel[c] as f32;
                            }
                            sum_weight[i] += weight;
                            max_weight[i] = weight.max(max_weight[i]);
                        }
                    }
                }
            }
        }
    }

    // Update the center pixels
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            if let Some(center_pixel) = padded_img.get_pixel_checked(x + offset_u32, y + offset_u32) {
                for c in 0..3 {
                    sum_image[i][c] += max_weight[i] * center_pixel[c] as f32;
                }
                sum_weight[i] += max_weight[i];
            }
        }
    }

    // Final image generation
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            if sum_weight[i] > 0.0 {
                let pixel = [
                    (sum_image[i][0] / sum_weight[i]).round().max(0.0).min(255.0) as u8,
                    (sum_image[i][1] / sum_weight[i]).round().max(0.0).min(255.0) as u8,
                    (sum_image[i][2] / sum_weight[i]).round().max(0.0).min(255.0) as u8,
                ];
                new_img.put_pixel(x, y, Rgb(pixel));
            } else {
                new_img.put_pixel(x, y, *img.get_pixel(x, y));
            }
        }
    }
}

fn total_variation(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    new_img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    _lambda: f32,
    _iterations: usize,
) {
    let mut u = vec![vec![vec![0.0f64; 3]; width as usize]; height as usize];
    let mut u0 = vec![vec![vec![0.0f64; 3]; width as usize]; height as usize];
    
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            for c in 0..3 {  // Add this loop to iterate over channels
                u[y as usize][x as usize][c] = pixel[c] as f64;
                u0[y as usize][x as usize][c] = pixel[c] as f64;
            }
        }
    }

    let h = 1.0; // Discrete spatial step
    let lambda = 0.1; // Regularization parameter
    let iter_max = 50; // Maximum iterations
    
    for _ in 0..iter_max {
        for c in 0..3 {  // Add this loop to iterate over channels
            for i in 1..height as usize - 1 {
                for j in 1..width as usize - 1 {
                    let mut ux = (u[i+1][j][c] - u[i][j][c]) / h;
                    let mut uy = (u[i][j+1][c] - u[i][j-1][c]) / (2.0 * h);
                    let mut grad_u = (ux * ux + uy * uy).sqrt();
                    let co1 = 1.0 / (grad_u + 1e-10); // Avoid division by zero
                    
                    ux = (u[i][j][c] - u[i-1][j][c]) / h;
                    uy = (u[i-1][j+1][c] - u[i-1][j-1][c]) / (2.0 * h);
                    grad_u = (ux * ux + uy * uy).sqrt();
                    let co2 = 1.0 / (grad_u + 1e-10);
                    
                    ux = (u[i+1][j][c] - u[i-1][j][c]) / (2.0 * h);
                    uy = (u[i][j+1][c] - u[i][j][c]) / h;
                    grad_u = (ux * ux + uy * uy).sqrt();
                    let co3 = 1.0 / (grad_u + 1e-10);
                    
                    ux = (u[i+1][j-1][c] - u[i-1][j-1][c]) / (2.0 * h);
                    uy = (u[i][j][c] - u[i][j-1][c]) / h;
                    grad_u = (ux * ux + uy * uy).sqrt();
                    let co4 = 1.0 / (grad_u + 1e-10);
                    
                    let numerator = u0[i][j][c] + (1.0 / (lambda * h * h)) * (
                        co1 * u[i+1][j][c] + 
                        co2 * u[i-1][j][c] + 
                        co3 * u[i][j+1][c] + 
                        co4 * u[i][j-1][c]
                    );
                    let denominator = 1.0 + (1.0 / (lambda * h * h)) * (co1 + co2 + co3 + co4);
                    u[i][j][c] = numerator / denominator;
                }
            }
        }
        
        for i in 1..height as usize - 1 {
            for c in 0..3 {  // Add this loop to iterate over channels
                u[i][0][c] = u[i][1][c];
                u[i][width as usize - 1][c] = u[i][width as usize - 2][c];
            }
        }
        
        for j in 1..width as usize - 1 {
            for c in 0..3 {  // Add this loop to iterate over channels
                u[0][j][c] = u[1][j][c];
                u[height as usize - 1][j][c] = u[height as usize - 2][j][c];
            }
        }
        
        for c in 0..3 {  // Add this loop to iterate over channels
            u[0][0][c] = u[1][1][c];
            u[0][width as usize - 1][c] = u[1][width as usize - 2][c];
            u[height as usize - 1][0][c] = u[height as usize - 2][1][c];
            u[height as usize - 1][width as usize - 1][c] = u[height as usize - 2][width as usize - 2][c];
        }
    }

    // Convert result back to image
    for y in 0..height {
        for x in 0..width {
            let pixel = [
                u[y as usize][x as usize][0].max(0.0).min(255.0) as u8,
                u[y as usize][x as usize][1].max(0.0).min(255.0) as u8,
                u[y as usize][x as usize][2].max(0.0).min(255.0) as u8,
            ];
            new_img.put_pixel(x, y, Rgb(pixel));
        }
    }
}

