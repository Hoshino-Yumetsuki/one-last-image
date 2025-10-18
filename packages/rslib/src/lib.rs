use serde::Deserialize;
use wasm_bindgen::prelude::*;

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
    // base64-encoded pencil texture image data (no data:* prefix)
    pub pencil_texture: Option<String>,
}

// One Last Image 主函数 - 将图片转换为线稿效果
// config_json 为可选参数，传入空字符串或无效JSON时使用默认配置
#[wasm_bindgen]
pub fn one_last_image(input: &[u8], config_json: Option<String>) -> Vec<u8> {
    let config = config_json.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            serde_json::from_str::<OLIConfig>(&s).ok()
        }
    });

    utils::image_processing::one_last_image_with_config(input, config)
}
