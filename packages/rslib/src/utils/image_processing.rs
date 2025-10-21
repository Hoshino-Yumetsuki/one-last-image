use base64::Engine;
use image::ImageEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, DynamicImage, GenericImageView, Rgba, RgbaImage};

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

// 简单的卷积函数（只处理Y通道）
fn convolve_y(pixels: &[u8], width: u32, height: u32, kernel: &[f32]) -> Vec<u8> {
    let side = (kernel.len() as f32).sqrt() as usize;
    let half = side / 2;
    let mut output = vec![0u8; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for ky in 0..side {
                for kx in 0..side {
                    let py =
                        (y as i32 + ky as i32 - half as i32).clamp(0, height as i32 - 1) as u32;
                    let px = (x as i32 + kx as i32 - half as i32).clamp(0, width as i32 - 1) as u32;
                    let idx = (py * width + px) as usize;
                    sum += pixels[idx] as f32 * kernel[ky * side + kx];
                }
            }
            output[(y * width + x) as usize] = sum.clamp(0.0, 255.0) as u8;
        }
    }
    output
}

// 生成高斯卷积核
fn gaussian_kernel(size: usize, sigma: f32) -> Vec<f32> {
    let half = (size / 2) as i32;
    let mut kernel = vec![0.0; size * size];
    let mut sum = 0.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - half;
            let dy = y as i32 - half;
            let value = (-((dx * dx + dy * dy) as f32) / (2.0 * sigma * sigma)).exp();
            kernel[y * size + x] = value;
            sum += value;
        }
    }

    // 归一化
    for val in kernel.iter_mut() {
        *val /= sum;
    }

    kernel
}

// Unsharp Mask 锐化
fn unsharp_mask(pixels: &[u8], width: u32, height: u32, amount: f32, radius: f32) -> Vec<u8> {
    // 使用高斯模糊
    let kernel = gaussian_kernel(5, radius);
    let blurred = convolve_y(pixels, width, height, &kernel);

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

// 从渐变色表中获取颜色
fn get_gradient_color(t: f32) -> (u8, u8, u8) {
    // 渐变色停靠点: (位置, R, G, B)
    const GRADIENT_STOPS: [(f32, f32, f32, f32); 6] = [
        (0.0, 251.0, 186.0, 48.0),
        (0.4, 252.0, 114.0, 53.0),
        (0.6, 252.0, 53.0, 78.0),
        (0.7, 207.0, 54.0, 223.0),
        (0.8, 55.0, 181.0, 217.0),
        (1.0, 62.0, 182.0, 218.0),
    ];

    // 找到t所在的区间并插值
    for i in 0..GRADIENT_STOPS.len() - 1 {
        let (pos1, r1, g1, b1) = GRADIENT_STOPS[i];
        let (pos2, r2, g2, b2) = GRADIENT_STOPS[i + 1];

        if t <= pos2 {
            let local_t = (t - pos1) / (pos2 - pos1);
            return (
                lerp(r1, r2, local_t) as u8,
                lerp(g1, g2, local_t) as u8,
                lerp(b1, b2, local_t) as u8,
            );
        }
    }

    // 超出范围则返回最后一个颜色
    let (_, r, g, b) = GRADIENT_STOPS[GRADIENT_STOPS.len() - 1];
    (r as u8, g as u8, b as u8)
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

    // 计算尺寸
    let (ori_w, ori_h) = img.dimensions();
    let mut width = (ori_w as f32 / zoom).round() as u32;
    let mut height = (ori_h as f32 / zoom).round() as u32;

    if width > 1920 {
        height = (height as f32 * 1920.0 / width as f32) as u32;
        width = 1920;
    }

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
        let kernel = gaussian_kernel(3, 0.8);
        gray = convolve_y(&gray, width, height, &kernel);
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

    // 6. 锐化边缘 - 使用Unsharp Mask增强边缘对比度
    // amount: 锐化强度, radius: 锐化半径
    processed = unsharp_mask(&processed, width, height, 1.5, 1.0);

    // 7. 生成最终RGBA图像
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

    // 8. 白色背景合成
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

    // 9. 水印（在白色背景合成之后）
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
