use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::ImageEncoder;
use image::{ColorType, DynamicImage, ImageBuffer, Luma, RgbaImage};
use imageproc::filter::filter3x3;
use imageproc::gradients::{horizontal_sobel, vertical_sobel};
use ndarray::Array2;
use image::GenericImageView;
use base64::Engine;

fn load_image(input: &[u8]) -> Option<DynamicImage> {
    image::load_from_memory(input).ok()
}

fn encode_png(rgba: &RgbaImage, ct: CompressionType) -> Option<Vec<u8>> {
    let (w, h) = rgba.dimensions();
    let mut out = Vec::new();
    let encoder = PngEncoder::new_with_quality(&mut out, ct, FilterType::Adaptive);
    match encoder.write_image(rgba.as_raw(), w, h, ColorType::Rgba8.into()) {
        Ok(_) => Some(out),
        Err(_) => None,
    }
}


// 高斯模糊滤波器 (3x3)
const DENOISE_KERNEL: [f32; 9] = [
    1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0,
    1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0,
    1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0,
];

// remove unused constants


// remove unused constants


/// One Last Image 主处理函数
/// 将图片转换为线稿效果
fn average_kernel(size: usize) -> Vec<f32> {
    vec![1.0 / (size * size) as f32; size * size]
}

fn emboss_kernel() -> Vec<f32> {
    // Simple emboss-like kernel
    vec![
        1.0, 1.0, 1.0,
        1.0, 1.0, -1.0,
        -1.0, -1.0, -1.0,
    ]
}

fn get_convolute_by_quality(quality: Option<&String>) -> Option<Vec<f32>> {
    match quality.map(|s| s.as_str()) {
        Some("fine") => Some(average_kernel(5)),
        Some("normal") => Some(average_kernel(7)),
        Some("coarse") => Some(average_kernel(9)),
        Some("superCoarse") => Some(average_kernel(11)),
        Some("extraCoarse") => Some(average_kernel(13)),
        Some("emboss") => Some(emboss_kernel()),
        Some("sketch") | None => None,
        _ => Some(average_kernel(7)),
    }
}

pub fn one_last_image_with_config(input: &[u8], config: Option<crate::OLIConfig>) -> Vec<u8> {
    let Some(img) = load_image(input) else {
        return input.to_vec();
    };

    let (width, height) = img.dimensions();

    // 使用配置项
    let dark_cut = config.as_ref().and_then(|c| c.dark_cut).unwrap_or(30) as f32;
    let light_cut = config.as_ref().and_then(|c| c.light_cut).unwrap_or(30) as f32;
    let denoise_enabled = config.as_ref().and_then(|c| c.denoise).unwrap_or(true);
    let _shade_enabled = config.as_ref().and_then(|c| c.shade).unwrap_or(true);
    let quality = config.as_ref().and_then(|c| c.quality.clone());
    let kiss = config.as_ref().and_then(|c| c.kiss).unwrap_or(false);
    let watermark = config.as_ref().and_then(|c| c.watermark).unwrap_or(false);
    let hajimei = config.as_ref().and_then(|c| c.hajimei).unwrap_or(false);
    let tone_count = config.as_ref().and_then(|c| c.tone_count).unwrap_or(3);
    let light = config.as_ref().and_then(|c| c.light).unwrap_or(0.0) as f32;

    // 第一步：转灰度
    let gray = img.to_luma8();

    // 第二步：去噪（使用 imageproc.filter3x3 for 3x3 kernel）
    let mut denoised = if denoise_enabled {
        filter3x3(&gray, &DENOISE_KERNEL)
    } else {
        gray
    };

    // 根据质量选择卷积矩阵
    let convolute_opt = get_convolute_by_quality(quality.as_ref());

    fn convolve_ndarray(
    pixels: &ImageBuffer<Luma<u8>, Vec<u8>>,
    weights: &[f32],
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let side = (weights.len() as f32).sqrt() as usize;
    if side * side != weights.len() {
        // invalid kernel
        return pixels.clone();
    }

    let (w, h) = pixels.dimensions();
    let mut output = ImageBuffer::new(w, h);

    // convert image to ndarray
    let mut arr = Array2::<f32>::zeros((h as usize, w as usize));
    for y in 0..h {
        for x in 0..w {
            arr[[y as usize, x as usize]] = pixels.get_pixel(x, y).0[0] as f32;
        }
    }

    let k = Array2::from_shape_vec((side, side), weights.to_vec()).unwrap();

    let pad = side / 2;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let mut sum = 0.0f32;
            for ky in 0..side {
                for kx in 0..side {
                    let iy = (y + ky).wrapping_sub(pad);
                    let ix = (x + kx).wrapping_sub(pad);
                    let iy_clamped = iy.min(h as usize - 1);
                    let ix_clamped = ix.min(w as usize - 1);
                    let pv = arr[[iy_clamped, ix_clamped]];
                    let wt = k[[ky, kx]];
                    sum += pv * wt;
                }
            }
            output.put_pixel(x as u32, y as u32, Luma([sum.max(0.0).min(255.0) as u8]));
        }
    }

    output
}

    // 计算边缘（使用卷积或 Sobel）
    let _edge_image = if let Some(weights) = &convolute_opt {
        if weights.len() == 9 {
            filter3x3(&denoised, weights.as_slice())
        } else {
            convolve_ndarray(&denoised, weights)
        }
    } else {
        let gx = horizontal_sobel(&denoised);
        let gy = vertical_sobel(&denoised);
        let (w2, h2) = denoised.dimensions();
        let mut mag = ImageBuffer::new(w2, h2);
        for yy in 0..h2 {
            for xx in 0..w2 {
                let a = gx.get_pixel(xx, yy).0[0] as f32;
                let b = gy.get_pixel(xx, yy).0[0] as f32;
                let v = ((a.abs() + b.abs()) / 2.0).min(255.0) as u8;
                mag.put_pixel(xx, yy, Luma([v]));
            }
        }
        mag
    };

    // 对比度增强
    let scale = 255.0 / (255.0 - light_cut - dark_cut);
    for pixel in denoised.pixels_mut() {
        let val = pixel.0[0] as f32;
        let enhanced = ((val - dark_cut) * scale + (val * (light / 100.0))).max(0.0).min(255.0) as u8;
        pixel.0[0] = enhanced;
    }

    // 缩小再放大平滑
    let small_w = (width as f32 / 1.4) as u32;
    let small_h = (height as f32 / 1.3) as u32;
    let small_img = image::imageops::resize(
        &denoised,
        small_w,
        small_h,
        image::imageops::FilterType::Triangle
    );
    let resized = image::imageops::resize(
        &small_img,
        width,
        height,
        image::imageops::FilterType::Triangle
    );

    // 转换为 RGBA 并应用 kiss 渐变（若启用）
    let mut rgba = RgbaImage::new(width, height);

    // prepare simple gradient for kiss
    let gradient = if kiss {
        // gradient from yellow -> orange -> magenta -> cyan
        Some(vec![
            (251u8, 186u8, 48u8),
            (252u8, 114u8, 53u8),
            (252u8, 53u8, 78u8),
            (207u8, 54u8, 223u8),
            (55u8, 181u8, 217u8),
        ])
    } else { None };

    for y in 0..height {
        for x in 0..width {
            let val = resized.get_pixel(x, y).0[0];
            if let Some(g) = gradient.as_ref() {
                // map val (0..255) to gradient and set alpha inversely
                let t = (val as f32 / 255.0) as f32;
                let idx_f = t * ((g.len() - 1) as f32);
                let idx = idx_f.floor() as usize;
                let frac = idx_f - (idx as f32);
                let c1 = g[idx];
                let c2 = g.get(idx + 1).copied().unwrap_or(c1);
                let r = (c1.0 as f32 * (1.0 - frac) + c2.0 as f32 * frac) as u8;
                let gcol = (c1.1 as f32 * (1.0 - frac) + c2.1 as f32 * frac) as u8;
                let b = (c1.2 as f32 * (1.0 - frac) + c2.2 as f32 * frac) as u8;
                let alpha = (255u8 as f32 * (1.0 - t)) as u8; // darker lines more opaque
                rgba.put_pixel(x, y, image::Rgba([r, gcol, b, alpha]));
            } else {
                // normal grayscale
                rgba.put_pixel(x, y, image::Rgba([val, val, val, 255]));
            }
        }
    }

    // 如果 watermark 为真，尝试使用内联的水印图片（base64）进行叠加
    // 在 Rust 层我们支持从 config 中通过 base64 字符串传入 watermarks
    if watermark {
        if let Some(cfg) = &config {
            if let Some(wm_b64) = cfg.watermark_image.as_ref() {
                // 解码 base64
                // use the new base64 engine API
if let Ok(wm_bytes) = base64::engine::general_purpose::STANDARD.decode(wm_b64) {
                    if let Ok(wm_img) = image::load_from_memory(&wm_bytes) {
                        let (wm_w, wm_h) = wm_img.dimensions();

                        // 根据 hajimei 标志选择裁切：如果 hajimei 为真，则使用 watermark 图片的上半部分作为“初回”样式
                        let wm_cropped = if hajimei && wm_h >= 2 {
                            let wm_rgba = wm_img.to_rgba8();
                            let half_h = wm_h / 2;
                            image::imageops::crop_imm(&wm_rgba, 0, 0, wm_w, half_h).to_image()
                        } else {
                            wm_img.to_rgba8()
                        };

                        let (used_w, used_h) = wm_cropped.dimensions();

                        // 将 watermark 缩放为原图的 30% 宽度（按比例）
                        let target_w = (width as f32 * 0.3) as u32;
                        let scale = target_w as f32 / used_w as f32;
                        let target_h = (used_h as f32 * scale) as u32;
                        let wm_resized = image::imageops::resize(&wm_cropped, target_w, target_h, image::imageops::FilterType::Triangle);

                        // 将 watermark 以右下角为原点叠加到 rgba 上
                        let start_x = width.saturating_sub(target_w + 10);
                        let start_y = height.saturating_sub(target_h + 10);
                        for yy in 0..target_h {
                            for xx in 0..target_w {
                                let dst_x = start_x + xx;
                                let dst_y = start_y + yy;
                                if dst_x < width && dst_y < height {
                                    let src_px = wm_resized.get_pixel(xx, yy).0;
                                    let dst_px = rgba.get_pixel(dst_x, dst_y).0;
                                    // alpha blend: src.a / 255 overlay on dst
                                    let alpha = src_px[3] as u32;
                                    if alpha > 0 {
                                        let inv_a = 255 - alpha;
                                        let r = ((src_px[0] as u32 * alpha + dst_px[0] as u32 * inv_a) / 255) as u8;
                                        let g = ((src_px[1] as u32 * alpha + dst_px[1] as u32 * inv_a) / 255) as u8;
                                        let b = ((src_px[2] as u32 * alpha + dst_px[2] as u32 * inv_a) / 255) as u8;
                                        let a = 255u8;
                                        rgba.put_pixel(dst_x, dst_y, image::Rgba([r, g, b, a]));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 如果需要，按 tone_count 执行调子分层（将灰度或边缘值量化为离散层）
    if tone_count > 1 {
        // 对刚才生成的 rgba（其 R/G/B 在非 kiss 模式下为相同灰度）进行量化处理
        for pixel in rgba.pixels_mut() {
            // 取亮度参考（在 kiss 模式下，可以用 R 通道近似）
            let l = pixel[0] as f32 / 255.0;
            let levels = (tone_count as f32).max(1.0);
            let q = ( (l * (levels - 1.0)).round() / (levels - 1.0) ).max(0.0).min(1.0);
            let v = (q * 255.0) as u8;
            pixel[0] = v;
            pixel[1] = v;
            pixel[2] = v;
            // keep alpha as-is
        }
    }

    // 编码为 PNG
    encode_png(&rgba, CompressionType::Default).unwrap_or_else(|| input.to_vec())
}


