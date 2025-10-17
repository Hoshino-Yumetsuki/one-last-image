use wasm_bindgen::prelude::*;
use serde::Deserialize;

mod utils;

#[derive(Deserialize)]
pub struct OLIConfig {
    pub zoom: Option<f32>,
    pub cover: Option<bool>,
    pub quality: Option<String>,
    pub denoise: Option<bool>,
    pub light_cut: Option<u8>,
    pub dark_cut: Option<u8>,
    pub shade: Option<bool>,
    pub shade_limit: Option<u8>,
    pub shade_light: Option<u8>,
    pub kiss: Option<bool>,
    pub watermark: Option<bool>,
    // base64-encoded watermark image data (no data:* prefix), e.g. one-last-image-logo2.png
    pub watermark_image: Option<String>,
    pub hajimei: Option<bool>,
    pub tone_count: Option<u8>,
    pub light: Option<f32>,
}

// One Last Image 主函数 - 将图片转换为线稿效果，支持传入 JSON 配置
#[wasm_bindgen]
pub fn one_last_image_with_config(input: &[u8], config_json: &str) -> Vec<u8> {
    let config: Result<OLIConfig, _> = serde_json::from_str(config_json);
    match config {
        Ok(cfg) => utils::image_processing::one_last_image_with_config(input, Some(cfg)),
        Err(_) => utils::image_processing::one_last_image_with_config(input, None),
    }
}

// 兼容老接口（不带配置）
#[wasm_bindgen]
pub fn one_last_image(input: &[u8]) -> Vec<u8> {
    utils::image_processing::one_last_image_with_config(input, None)
}

//#[wasm_bindgen]
//pub fn detect_mime(input: &[u8]) -> String {
//    utils::mime::detect_mime(input)
//}
