use palette::encoding::srgb;
use palette::{hsv, Hsv, Srgb};
use palette::convert::IntoColorUnclamped;

pub fn float_to_rgb_palette(f: f32) -> (u8, u8, u8) {
    //正規化
    let clamped_value = f.max(0.1).min(1.0);
    let normalized_value = (clamped_value - 0.1) / (1.0 - 0.1);

    //Hue 0.0 -> 300.0度にマッピング
    //0.0は赤、60.0は黄、120.0は緑、180.0はシアン、240.0は青、300.0はマゼンタの手前
    let hue = normalized_value * 300.0;
    
    //彩度 max
    let saturation = 1.0;
    //明度 max
    let value = 1.0;
     
    let hsv_color = Hsv::new(hue, saturation, value);
    
    //HSVからsRGBに変換
    let srgb_color: Srgb = hsv_color.into_color_unclamped();

    let r = (srgb_color.red * 255.0).round() as u8;
    let g = (srgb_color.green * 255.0).round() as u8;
    let b = (srgb_color.blue * 255.0).round() as u8;

    (r, g, b)

}