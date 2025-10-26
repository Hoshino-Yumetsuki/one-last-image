use base64::Engine;
use image::ImageEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, DynamicImage, GenericImageView, Rgba, RgbaImage};
use ndarray::Array2;
use rayon::prelude::*;

fn load_image(input: &[u8]) -> Option<DynamicImage> {
    image::load_from_memory(input).ok()
}

fn encode_png(rgba: &RgbaImage) -> Option<Vec<u8>> {
    let (w, h) = rgba.dimensions();
    let mut out = Vec::new();
    let encoder =
        PngEncoder::new_with_quality(&mut out, CompressionType::Fast, FilterType::Adaptive);
    match encoder.write_image(rgba.as_raw(), w, h, ColorType::Rgba8.into()) {
        Ok(_) => Some(out),
        Err(_) => None,
    }
}

// 使用ndarray进行卷积运算（并行优化版本）
fn convolve_ndarray(pixels: &[u8], width: u32, height: u32, kernel: &Array2<f32>) -> Vec<u8> {
    let (kh, kw) = kernel.dim();
    let half_h = kh / 2;
    let half_w = kw / 2;

    // 使用并行迭代器处理每一行
    let output: Vec<u8> = (0..height)
        .into_par_iter()
        .flat_map(|y| {
            let mut row = vec![0u8; width as usize];
            for x in 0..width {
                let mut sum = 0.0;
                for ky in 0..kh {
                    for kx in 0..kw {
                        let py = (y as i32 + ky as i32 - half_h as i32).clamp(0, height as i32 - 1)
                            as u32;
                        let px = (x as i32 + kx as i32 - half_w as i32).clamp(0, width as i32 - 1)
                            as u32;
                        let idx = (py * width + px) as usize;
                        sum += pixels[idx] as f32 * kernel[[ky, kx]];
                    }
                }
                row[x as usize] = sum.clamp(0.0, 255.0) as u8;
            }
            row
        })
        .collect();

    output
}

// 简单的卷积函数（兼容旧代码）
fn convolve_y(pixels: &[u8], width: u32, height: u32, kernel: &[f32]) -> Vec<u8> {
    let side = (kernel.len() as f32).sqrt() as usize;
    let kernel_array = Array2::from_shape_vec((side, side), kernel.to_vec()).unwrap();
    convolve_ndarray(pixels, width, height, &kernel_array)
}

// 使用ndarray生成高斯卷积核
fn gaussian_kernel_ndarray(size: usize, sigma: f32) -> Array2<f32> {
    let half = (size / 2) as i32;
    let mut kernel = Array2::zeros((size, size));
    let mut sum = 0.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - half;
            let dy = y as i32 - half;
            let value = (-((dx * dx + dy * dy) as f32) / (2.0 * sigma * sigma)).exp();
            kernel[[y, x]] = value;
            sum += value;
        }
    }

    // 归一化
    kernel / sum
}

// 动态生成Sobel算子 - Sobel = 高斯平滑 ⊗ 差分
// Sobel算子是[1,2,1]^T（高斯平滑）和[-1,0,1]（差分）的外积
fn sobel_x_kernel() -> Array2<f32> {
    // 高斯平滑向量（垂直方向）
    let smooth = Array2::from_shape_vec((3, 1), vec![1.0, 2.0, 1.0]).unwrap();
    // 差分向量（水平方向）
    let diff = Array2::from_shape_vec((1, 3), vec![-1.0, 0.0, 1.0]).unwrap();

    // 外积生成Sobel X算子
    let mut kernel = Array2::zeros((3, 3));
    for i in 0..3 {
        for j in 0..3 {
            kernel[[i, j]] = smooth[[i, 0]] * diff[[0, j]];
        }
    }
    kernel
}

fn sobel_y_kernel() -> Array2<f32> {
    // 差分向量（垂直方向）
    let diff = Array2::from_shape_vec((3, 1), vec![-1.0, 0.0, 1.0]).unwrap();
    // 高斯平滑向量（水平方向）
    let smooth = Array2::from_shape_vec((1, 3), vec![1.0, 2.0, 1.0]).unwrap();

    // 外积生成Sobel Y算子
    let mut kernel = Array2::zeros((3, 3));
    for i in 0..3 {
        for j in 0..3 {
            kernel[[i, j]] = diff[[i, 0]] * smooth[[0, j]];
        }
    }
    kernel
}

// Unsharp Mask 锐化（使用ndarray）
fn unsharp_mask(pixels: &[u8], width: u32, height: u32, amount: f32, radius: f32) -> Vec<u8> {
    // 使用高斯模糊
    let kernel = gaussian_kernel_ndarray(5, radius);
    let blurred = convolve_ndarray(pixels, width, height, &kernel);

    let mut output = vec![0u8; (width * height) as usize];
    for i in 0..output.len() {
        let original = pixels[i] as f32;
        let blur = blurred[i] as f32;
        let sharpened = original + amount * (original - blur);
        output[i] = sharpened.clamp(0.0, 255.0) as u8;
    }

    output
}

// 线性插值
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// 渐变色停靠点
const ORIGINAL_GRADIENT_STOPS: [(f32, u8, u8, u8); 6] = [
    (0.0, 251, 186, 48), // 橙黄色
    (0.4, 252, 114, 53), // 橙红色
    (0.6, 252, 53, 78),  // 粉红色
    (0.7, 207, 54, 223), // 紫红色
    (0.8, 55, 181, 217), // 青蓝色
    (1.0, 62, 182, 218), // 青蓝色
];

// 从原始渐变色表中获取颜色（使用线性插值）
fn get_gradient_color(t: f32) -> (u8, u8, u8) {
    let clamped_t = t.clamp(0.0, 1.0);

    // 找到t所在的区间并插值
    for i in 0..(ORIGINAL_GRADIENT_STOPS.len() - 1) {
        let (pos1, r1, g1, b1) = ORIGINAL_GRADIENT_STOPS[i];
        let (pos2, r2, g2, b2) = ORIGINAL_GRADIENT_STOPS[i + 1];

        if clamped_t <= pos2 {
            let local_t = (clamped_t - pos1) / (pos2 - pos1);
            return (
                lerp(r1 as f32, r2 as f32, local_t) as u8,
                lerp(g1 as f32, g2 as f32, local_t) as u8,
                lerp(b1 as f32, b2 as f32, local_t) as u8,
            );
        }
    }

    // 超出范围则返回最后一个颜色
    let (_, r, g, b) = ORIGINAL_GRADIENT_STOPS[ORIGINAL_GRADIENT_STOPS.len() - 1];
    (r, g, b)
}

// SMAA (Subpixel Morphological Anti-Aliasing) 实现
fn smaa_antialiasing(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    // 第一步：边缘检测
    let edges = smaa_edge_detection(pixels, width, height);

    // 第二步：混合权重计算
    let blend_weights = smaa_blend_weights(&edges, width, height);

    // 第三步：邻域混合
    smaa_neighborhood_blending(pixels, &blend_weights, width, height)
}

// SMAA 第一步：使用Sobel算子进行边缘检测
fn smaa_edge_detection(pixels: &[u8], width: u32, height: u32) -> Vec<f32> {
    let sobel_x = sobel_x_kernel();
    let sobel_y = sobel_y_kernel();

    let gx = convolve_ndarray(pixels, width, height, &sobel_x);
    let gy = convolve_ndarray(pixels, width, height, &sobel_y);

    // 并行计算梯度幅值
    let edges: Vec<f32> = gx
        .par_iter()
        .zip(gy.par_iter())
        .map(|(&gx_val, &gy_val)| {
            let gx_f = gx_val as f32;
            let gy_f = gy_val as f32;
            (gx_f * gx_f + gy_f * gy_f).sqrt()
        })
        .collect();

    edges
}

// SMAA 第二步：计算混合权重
fn smaa_blend_weights(edges: &[f32], width: u32, height: u32) -> Vec<f32> {
    let threshold = 20.0; // 边缘阈值

    // 并行处理每一行
    let weights: Vec<f32> = (1..(height - 1))
        .into_par_iter()
        .flat_map(|y| {
            let mut row = vec![0.0f32; width as usize];

            for x in 1..(width - 1) {
                let idx = (y * width + x) as usize;
                let edge_strength = edges[idx];

                if edge_strength > threshold {
                    // 计算局部边缘模式
                    let mut pattern_weight = 0.0;
                    let mut count = 0.0;

                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let ny = (y as i32 + dy) as u32;
                            let nx = (x as i32 + dx) as u32;
                            let nidx = (ny * width + nx) as usize;

                            if edges[nidx] > threshold {
                                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                                let weight = 1.0 / (1.0 + dist);
                                pattern_weight += weight;
                                count += 1.0;
                            }
                        }
                    }

                    // 归一化权重
                    row[x as usize] = if count > 0.0 {
                        (pattern_weight / count).min(1.0)
                    } else {
                        0.0
                    };
                }
            }
            row
        })
        .collect();

    // 添加首尾行（全零）
    let mut result = vec![0.0f32; width as usize];
    result.extend(weights);
    result.extend(vec![0.0f32; width as usize]);
    result
}

// SMAA 第三步：邻域混合
fn smaa_neighborhood_blending(pixels: &[u8], weights: &[f32], width: u32, height: u32) -> Vec<u8> {
    // 并行处理每一行
    let output: Vec<u8> = (0..height)
        .into_par_iter()
        .flat_map(|y| {
            let mut row = vec![0u8; width as usize];

            for x in 0..width {
                let idx = (y * width + x) as usize;
                let weight = weights[idx];

                if weight < 0.01 {
                    row[x as usize] = pixels[idx];
                } else {
                    // 使用双线性插值进行亚像素混合
                    let mut sum = pixels[idx] as f32 * (1.0 - weight);
                    let mut total_weight = 1.0 - weight;

                    // 采样周围像素
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }

                            let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as u32;
                            let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as u32;
                            let nidx = (ny * width + nx) as usize;

                            let dist = ((dx * dx + dy * dy) as f32).sqrt();
                            let sample_weight = weight / (1.0 + dist * 2.0);

                            sum += pixels[nidx] as f32 * sample_weight;
                            total_weight += sample_weight;
                        }
                    }

                    row[x as usize] = (sum / total_weight).clamp(0.0, 255.0) as u8;
                }
            }
            row
        })
        .collect();

    output
}

// 获取卷积核
fn get_kernel(quality: &str) -> Option<Vec<f32>> {
    match quality {
        "fine" => {
            let size = 5;
            Some(vec![1.0 / (size * size) as f32; size * size])
        }
        "normal" => {
            let size = 7;
            Some(vec![1.0 / (size * size) as f32; size * size])
        }
        "coarse" => {
            let size = 9;
            Some(vec![1.0 / (size * size) as f32; size * size])
        }
        "superCoarse" => {
            let size = 11;
            Some(vec![1.0 / (size * size) as f32; size * size])
        }
        "extraCoarse" => {
            let size = 13;
            Some(vec![1.0 / (size * size) as f32; size * size])
        }
        "sketch" => None,
        _ => {
            let size = 7;
            Some(vec![1.0 / (size * size) as f32; size * size])
        }
    }
}

pub fn one_last_image_with_config(input: &[u8], config: Option<crate::OLIConfig>) -> Vec<u8> {
    let Some(img) = load_image(input) else {
        return input.to_vec();
    };

    // 读取配置
    let zoom = config.as_ref().and_then(|c| c.zoom).unwrap_or(1.0);
    let quality = config
        .as_ref()
        .and_then(|c| c.quality.clone())
        .unwrap_or_else(|| "normal".to_string());
    let denoise = config.as_ref().and_then(|c| c.denoise).unwrap_or(true);
    let light = config.as_ref().and_then(|c| c.light).unwrap_or(0.0);
    let light_cut = config.as_ref().and_then(|c| c.light_cut).unwrap_or(128) as f32;
    let dark_cut = config.as_ref().and_then(|c| c.dark_cut).unwrap_or(118) as f32;
    let kiss = config.as_ref().and_then(|c| c.kiss).unwrap_or(true);
    let watermark = config.as_ref().and_then(|c| c.watermark).unwrap_or(true);
    let hajimei = config.as_ref().and_then(|c| c.hajimei).unwrap_or(false);

    // 计算尺寸 - 直接使用原始分辨率，不做限制
    let (ori_w, ori_h) = img.dimensions();
    let width = (ori_w as f32 / zoom).round() as u32;
    let height = (ori_h as f32 / zoom).round() as u32;

    // 缩放图片 - 使用Lanczos3获得更锐利的边缘
    let resized =
        image::imageops::resize(&img, width, height, image::imageops::FilterType::Lanczos3);

    // 1. 转灰度
    let mut gray = vec![0u8; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel = resized.get_pixel(x, y);
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;
            let luma = (r * 0.299 + g * 0.587 + b * 0.114).floor() as u8;
            gray[(y * width + x) as usize] = luma;
        }
    }

    // 2. light调整
    if light != 0.0 {
        for val in gray.iter_mut() {
            let v = *val as f32;
            *val = (v + v * (light / 100.0)).clamp(0.0, 255.0) as u8;
        }
    }

    // 3. 去噪 - 使用高斯滤波保留更多边缘细节
    if denoise {
        let kernel = gaussian_kernel_ndarray(3, 0.8);
        gray = convolve_ndarray(&gray, width, height, &kernel);
    }

    // 4. 卷积
    let kernel_opt = get_kernel(&quality);
    let mut processed = if let Some(ref kernel) = kernel_opt {
        let original = gray.clone();
        let convolved = convolve_y(&gray, width, height, &kernel);

        // 高通滤波: 128 + 原始 - 卷积
        let mut diff = vec![0u8; (width * height) as usize];
        for i in 0..diff.len() {
            let val = 128.0 + original[i] as f32 - convolved[i] as f32;
            diff[i] = val.clamp(0.0, 255.0) as u8;
        }
        diff
    } else {
        gray.clone()
    };

    // 5. lightCut/darkCut
    if kernel_opt.is_some() && (light_cut > 0.0 || dark_cut > 0.0) {
        let scale = 255.0 / (255.0 - light_cut - dark_cut);
        for val in processed.iter_mut() {
            let v = (*val as f32 - dark_cut) * scale;
            *val = v.clamp(0.0, 255.0) as u8;
        }
    }

    // 6. SMAA抗锯齿 - 高质量形态学抗锯齿
    processed = smaa_antialiasing(&processed, width, height);

    // 7. 适度锐化 - 保持细节清晰
    processed = unsharp_mask(&processed, width, height, 1.0, 0.9);

    // 8. 生成最终RGBA图像
    let mut rgba = RgbaImage::new(width, height);

    if kiss {
        // Kiss模式：彩色渐变
        for y in 0..height {
            for x in 0..width {
                let val = processed[(y * width + x) as usize];

                // 计算渐变位置
                let t = ((x as f32 + y as f32) / (width as f32 + height as f32)).min(1.0);

                // 从渐变色表获取颜色
                let (r, g, b) = get_gradient_color(t);

                // alpha = 255 - y（越暗越不透明）
                let alpha = (255.0 - val as f32) as u8;

                rgba.put_pixel(x, y, Rgba([r, g, b, alpha]));
            }
        }
    } else {
        // 灰度模式
        for y in 0..height {
            for x in 0..width {
                let val = processed[(y * width + x) as usize];
                rgba.put_pixel(x, y, Rgba([val, val, val, 255]));
            }
        }
    }

    // 9. 白色背景合成
    let mut final_img = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let src = rgba.get_pixel(x, y);
            let alpha = src[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;

            let r = (src[0] as f32 * alpha + 255.0 * inv_alpha).min(255.0) as u8;
            let g = (src[1] as f32 * alpha + 255.0 * inv_alpha).min(255.0) as u8;
            let b = (src[2] as f32 * alpha + 255.0 * inv_alpha).min(255.0) as u8;

            final_img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // 10. 水印（在白色背景合成之后）
    if watermark {
        if let Some(cfg) = &config {
            if let Some(wm_b64) = cfg.watermark_image.as_ref() {
                if let Ok(wm_bytes) = base64::engine::general_purpose::STANDARD.decode(wm_b64) {
                    if let Ok(wm_img) = image::load_from_memory(&wm_bytes) {
                        let (wm_w, wm_h) = wm_img.dimensions();
                        let wm_rgba = wm_img.to_rgba8();
                        let half_h = wm_h / 2;

                        let wm_cropped = if hajimei {
                            image::imageops::crop_imm(&wm_rgba, 0, half_h, wm_w, half_h).to_image()
                        } else {
                            image::imageops::crop_imm(&wm_rgba, 0, 0, wm_w, half_h).to_image()
                        };

                        let (used_w, used_h) = wm_cropped.dimensions();
                        let (set_width, set_height) = if width as f32 / height as f32 > 1.1 {
                            let h = (height as f32 * 0.15) as u32;
                            let w = (h as f32 / used_h as f32 * used_w as f32) as u32;
                            (w, h)
                        } else {
                            let w = (width as f32 * 0.3) as u32;
                            let h = (w as f32 / used_w as f32 * used_h as f32) as u32;
                            (w, h)
                        };

                        let wm_resized = image::imageops::resize(
                            &wm_cropped,
                            set_width,
                            set_height,
                            image::imageops::FilterType::Lanczos3,
                        );
                        let start_x =
                            width.saturating_sub(set_width + (set_height as f32 * 0.2) as u32);
                        let start_y =
                            height.saturating_sub(set_height + (set_height as f32 * 0.16) as u32);

                        for yy in 0..set_height {
                            for xx in 0..set_width {
                                let dst_x = start_x + xx;
                                let dst_y = start_y + yy;
                                if dst_x < width && dst_y < height {
                                    let src_px = wm_resized.get_pixel(xx, yy);
                                    let dst_px = final_img.get_pixel(dst_x, dst_y);
                                    let alpha = src_px[3] as u32;
                                    if alpha > 0 {
                                        let inv_a = 255 - alpha;
                                        let r = ((src_px[0] as u32 * alpha
                                            + dst_px[0] as u32 * inv_a)
                                            / 255)
                                            as u8;
                                        let g = ((src_px[1] as u32 * alpha
                                            + dst_px[1] as u32 * inv_a)
                                            / 255)
                                            as u8;
                                        let b = ((src_px[2] as u32 * alpha
                                            + dst_px[2] as u32 * inv_a)
                                            / 255)
                                            as u8;
                                        final_img.put_pixel(dst_x, dst_y, Rgba([r, g, b, 255]));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    encode_png(&final_img).unwrap_or_else(|| input.to_vec())
}
