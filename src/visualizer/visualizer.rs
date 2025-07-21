
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Widget},
    style::{Style, Color},
};

use crate::audio::analyzer::SpectrumData;
use crate::visualizer::visualize_color;

pub struct SpectrumVisualizer;

impl SpectrumVisualizer {

    pub fn new() -> Self {
        SpectrumVisualizer {}
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, spectrum_data: Option<&SpectrumData>) {

        //ヴィジュアライザーのブロックを作成
        let block = Block::default()
        .borders(Borders::ALL)
        .title("Audio Visualizer");

        frame.render_widget(&block, area);

        //棒グラフを描画する内部の描画エリアを計算
        let inner_area = block.inner(area);

        if let Some(data) = spectrum_data {
            let raw_bins = &data.bins;
            if raw_bins.is_empty() {
                return;
            }

            let num_display_bars = inner_area.width as usize;

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

            let max_height = inner_area.height as f32;

            for (i, &magnitude) in bins_to_process.iter().enumerate() {

                let x = inner_area.left() + i as u16; //各種1文字幅
                if x >= inner_area.right() { continue; } //描画領域超えたらスキップ

                let bar_height = (magnitude * max_height).min(max_height) as u16;
                let y = inner_area.bottom().saturating_sub(bar_height);

                let color = get_bar_color(i, bins_to_process.len());

                //角棒を描画
                for h in 0..bar_height {
                    frame.buffer_mut().set_style(Rect::new(x, y + h, 1, 1), Style::default().bg(color));
                }
            }
        }
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
