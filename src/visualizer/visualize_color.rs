use palette::encoding::srgb;
use palette::{hsv, Hsv, Srgb};
use palette::convert::IntoColorUnclamped;
use ratatui::style::Color;

const GRAY_FACTOR: f32 = 0.4;

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

pub fn get_lighter_color(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let factor = 0.5; // この値を小さくするとより薄く、大きくすると元の色に近くなる
            Color::Rgb(
                (r as f32 * factor + 255.0 * (1.0 - factor)) as u8,
                (g as f32 * factor + 255.0 * (1.0 - factor)) as u8,
                (b as f32 * factor + 255.0 * (1.0 - factor)) as u8,)
        },
        // その他の色タイプの場合、デフォルトの色を返すか、エラー処理
        _ => Color::Gray, // RGB以外の色には単純な灰色を返す
    }
}

//灰色成分
pub fn get_grayish_color(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let factor = GRAY_FACTOR;
            Color::Rgb(
                (r as f32 * factor + 128.0 * (1.0 - factor)) as u8,
                (g as f32 * factor + 128.0 * (1.0 - factor)) as u8,
                (b as f32 * factor + 128.0 * (1.0 - factor)) as u8,)
        },
        _ => Color::DarkGray,
    }
}