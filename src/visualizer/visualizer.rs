
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Widget},
    style::{Style, Color},
};

use crate::{audio::analyzer::SpectrumData, visualizer::{self, visualize_color::get_grayish_color}};
use crate::visualizer::visualize_color;

pub struct SpectrumVisualizer {
    prev_bar_heights: Vec<u16>,
}

const MIN_DB: f32 = -60.0;
const MAX_DB: f32 = 0.0;
const DB_RANGE: f32 = MAX_DB - MIN_DB;


impl SpectrumVisualizer {

    pub fn new() -> Self {
        SpectrumVisualizer {
            prev_bar_heights: Vec::new(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, spectrum_data: Option<&SpectrumData>) {

        //ヴィジュアライザーのブロックを作成
        let block = Block::default()
        .borders(Borders::ALL)
        .title("Audio Visualizer");

        frame.render_widget(&block, area);

        let full_area = frame.area();

        //棒グラフを描画する内部の描画エリアを計算
        let visualizer_width_percentage = 0.80;
        let visualizer_height_percentage = 0.50;

        let visualizer_width = (full_area.width as f32 * visualizer_width_percentage) as u16;
        let visualizer_height = (full_area.height as f32 * visualizer_height_percentage) as u16;

        let visualizer_x = full_area.left() + (full_area.width.saturating_sub(visualizer_width)) / 2;
        let visualizer_y = full_area.top() + (full_area.height.saturating_sub(visualizer_height)) / 2;

        let visualizer_area = Rect::new(visualizer_x, visualizer_y, visualizer_width, visualizer_height);

        let num_display_bars = visualizer_area.width as usize;
        let mut current_bar_heights = vec![0u16; num_display_bars];

        if let Some(data) = spectrum_data {

            let raw_bins = &data.bins;
            if raw_bins.is_empty() {
                return;
            }

            let bins_to_process = if raw_bins.len() <= num_display_bars {
                raw_bins.clone()
            } else {

                let mut aggregated_bins = vec![0.0f32; num_display_bars];
                let raw_bins_per_display_bin = raw_bins.len() as f32 / num_display_bars as f32;

                for i in 0..num_display_bars {

                    let start_index = (i as f32 * raw_bins_per_display_bin) as usize;
                    let end_index = (((i + 1) as f32 * raw_bins_per_display_bin) as usize).min(raw_bins.len());

                    let mut sum_magnitude = 0.0;
                    let mut count = 0;

                    for j in start_index..end_index {
                        sum_magnitude += raw_bins[j];
                        count += 1;
                    }

                    if count > 0 {
                        aggregated_bins[i] = sum_magnitude / count as f32;
                    }
                }
                aggregated_bins
            };

            let max_height = visualizer_area.height as f32;

            if self.prev_bar_heights.len() != num_display_bars {
                self.prev_bar_heights.resize(num_display_bars, 0);
            }

            for i in 0..num_display_bars {
                let x = visualizer_area.left() + i as u16;
                if x >= visualizer_area.right() { continue; }

                let prev_height = self.prev_bar_heights[i];
                let y_prev = visualizer_area.bottom().saturating_sub(prev_height);
                let original_color = get_bar_color(i, num_display_bars);
                let grayish_color = get_grayish_color(original_color);

                for h in 0..prev_height {
                    frame.buffer_mut().set_style(Rect::new(x, y_prev + h, 1, 1), Style::default().bg(grayish_color));
                }
            }

            for (i, &magnitude) in bins_to_process.iter().enumerate() {

                let x = visualizer_area.left() + i as u16; //各種1文字幅
                if x >= visualizer_area.right() { continue; } //描画領域超えたらスキップ

                let mut scaled_magnitude = 0.0;

                if magnitude > 0.0 {
                    //デシベル値に変換
                    let db_value = 20.0 * magnitude.log10();
                    //デシベル値をminからmaxの範囲で正規化
                    scaled_magnitude = ((db_value - MIN_DB) /DB_RANGE).max(0.0).min(1.0);
                }

                //正規化された振幅を最大高さにマッピング
                let bar_height_float = (scaled_magnitude * max_height).min(max_height);

                let mut bar_height = bar_height_float as u16;
                if bar_height == 0 && magnitude > 0.0 {
                    bar_height = 1;
                }

                current_bar_heights[i] = bar_height;

                let y = visualizer_area.bottom().saturating_sub(bar_height);

                let color = get_bar_color(i, bins_to_process.len());

                //角棒を描画
                for h in 0..bar_height {
                    frame.buffer_mut().set_style(Rect::new(x, y + h, 1, 1), Style::default().bg(color));
                }
            }
        }

        self.prev_bar_heights = current_bar_heights;

    }

    #[cfg(test)]
    pub(crate) fn aggregated_bins(raw_bins: &[f32], target_count: usize) -> Vec<f32> {

        if raw_bins.is_empty() || target_count == 0 {
            return Vec::new();
        }
        if raw_bins.len() <= target_count {
            return raw_bins.to_vec();
        }

        let mut aggregated_bins = vec![0.0f32; target_count];
        let raw_bins_per_display_bin = raw_bins.len() as f32 / target_count as f32;

        for i in 0..target_count {
            let start_index = (i as f32 * raw_bins_per_display_bin) as usize;
            let end_index = (((i + 1) as f32 * raw_bins_per_display_bin) as usize).min(raw_bins.len());

            let mut sum_magnitude = 0.0;
            let mut count = 0;
            for j in start_index..end_index {
                sum_magnitude += raw_bins[j];
                count += 1; 
            }

            if count > 0 {
                aggregated_bins[i] = sum_magnitude / count as f32;
            }
        }
        aggregated_bins

    }

}

fn get_bar_color(index: usize, total_bars: usize) -> Color {
    let ratio = index as f32 / total_bars as f32;
    let rgb = visualize_color::float_to_rgb_palette(ratio);
    
    Color::Rgb(rgb.0, rgb.1, rgb.2)
}


#[cfg(test)]
mod test_visualiezr {
    use std::vec;

    use crate::visualizer::visualizer::SpectrumVisualizer;


    #[test]
    fn test_aggregate_bins_basic() {
        let raw_bins = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let target_count = 4; //2つの生ビンが1つの表示ビンに集約されるはず

        let aggregated = SpectrumVisualizer::aggregated_bins(&raw_bins, target_count);

        assert_eq!(aggregated.len(), target_count);

        //各集約ビンの値が平均値になっていることを確認
        assert_eq!(aggregated[0], (1.0 + 2.0) / 2.0);
        assert_eq!(aggregated[1], (3.0 + 4.0) / 2.0);
        assert_eq!(aggregated[2], (5.0 + 6.0) / 2.0);
        assert_eq!(aggregated[3], (7.0 + 8.0) / 2.0);
    }

    #[test]
    fn test_aggregate_bins_uneven() {
        let raw_bins = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let target_count = 2;

        let aggregated = SpectrumVisualizer::aggregated_bins(&raw_bins, target_count);

        assert_eq!(aggregated.len(), target_count);
        assert_eq!(aggregated[0], (1.0 + 2.0) / 2.0);
        assert_eq!(aggregated[1], (3.0 + 4.0 + 5.0) / 3.0);
    }

    #[test]
    fn test_aggregated_bins_less_than_target() {
        let raw_bins = vec![1.0, 2.0, 3.0];
        let target_count = 5;

        let aggregated = SpectrumVisualizer::aggregated_bins(&raw_bins, target_count);
        assert_eq!(aggregated.len(), raw_bins.len());
        assert_eq!(aggregated, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_aggregated_bins_empty_input() {
        let raw_bins: Vec<f32> = Vec::new();
        let target_count = 10;
        let aggregated = SpectrumVisualizer::aggregated_bins(&raw_bins, target_count);
        assert!(aggregated.is_empty());
    }

    #[test]
    fn test_agregate_bins_zero_target() {
        let raw_bins = vec![1.0, 2.0, 3.0];
        let target_count = 0;
        let aggregated = SpectrumVisualizer::aggregated_bins(&raw_bins, target_count);
        assert!(aggregated.is_empty());
    }

}
