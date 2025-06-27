
use realfft::num_complex::Complex;
use realfft::{RealFftPlanner, ComplexToReal};
use rodio::buffer::SamplesBuffer;
use rodio::cpal::SampleRate;
use std::vec::Vec;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::audio::analyzer;
use crate::error::FerriaError;

//オーディオサンプルを分析して、周波数スペクトルを生成
pub struct AudioAnalyzer {
    fft_size: usize,
    sample_rate: u32,
    planner: RealFftPlanner<f32>,
}

//分析結果として得られるスペクトルデータ
#[derive(Debug, Clone)]
pub struct SpectrumData {
    pub bands: Vec<f32>,
    pub max_amplitude: f32,
}

impl AudioAnalyzer {

    pub fn new(fft_size: usize, sample_rate: u32) -> Result<Self, FerriaError> {

        if !fft_size.is_power_of_two() || fft_size == 0 {
            return Err(FerriaError::AnalyzerError(format!("FFT size must be a power of two and non-zero, got {}", fft_size)));
        }

        Ok( AudioAnalyzer{
            fft_size,
            sample_rate,
            planner: RealFftPlanner::<f32>::new(),
            } )

    }

    ///オーディオサンプルから周波数スペクトルを計算する
    ///inline展開を試してみる
    /// 
    //オーディオサンプルにHann窓関数を適応
    #[inline]
    fn apply_window_function(&self, samples: &mut [f32]) {

        let n = samples.len() as f32;

        if n <= 1.0 {
            //1サンプル以下なら窓関数は不要
            return;
        }

        for i in 0..samples.len() {

            let  window_val = 0.5f32 * (1.0f32 - (2.0f32 * std::f32::consts::PI * i as f32 / (n - 1.0f32)).cos());

            samples[i] *= window_val;

        }

    }

    #[inline]
    //FFT結果の複素数から正規化された振幅スペクトルデータを計算する
    fn calculate_spectrum_data(&self, fft_output: &[Complex<f32>]) -> SpectrumData {

        let mut max_amplitude = 0.0f32;

        let mut bands: Vec<f32> = fft_output.iter()
        .skip(1)//最初の要素(DC成分)と最後の要素(ナイキスト周波数)は通常は除外
        .map(|c| {

            let amp = c.norm();//複素数の絶対値(振幅)

            if amp > max_amplitude {
                max_amplitude = amp;
            }

            amp

        })
        .collect();


        if max_amplitude > 0.0 {
            //全体の最大振幅で正規化(表示レンジに合わせるため)
            for band in bands.iter_mut() {
                *band /= max_amplitude;
            }
        }
        else {
            //max_amplitudeが0の場合(無音など),全てのバンドも0にする
            for band in bands.iter_mut() {
                *band = 0.0;
            }
        }

        //ここで必要に応じて、対数スケールへの変換や、周波数帯域のグルーピング(ex: オクターブバンド)を行う
        //現状は線形スケールの振幅をそのまま返す

        SpectrumData { 
            bands, 
            max_amplitude,
        }

    } 

    #[inline]
    pub fn analyze(&mut self, samples: &[f32]) -> Result<SpectrumData, FerriaError> {

        if samples.len() != self.fft_size {
            return Err(FerriaError::AnalyzerError(format!("Input sample slice length({}) does not match FFT size ({})", samples.len(), self.fft_size)));
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);

        let mut input_buffer = samples.to_vec();

        self.apply_window_function(&mut input_buffer);

        //FFT結果の出力バッファ
        let mut output_buffer = vec![Complex::new(0.0, 0.0); self.fft_size / 2 + 1];

        fft.process(&mut input_buffer, &mut output_buffer)
        .map_err(|e| FerriaError::AnalyzerError(format!("FFT processing failed: {}", e)))?;

        Ok(self.calculate_spectrum_data(&output_buffer))

    }

    pub fn run_in_thread (
        fft_size: usize,
        sample_rate: u32,
        sample_rx: mpsc::Receiver<f32>,
        spectrum_tx: mpsc::Sender<SpectrumData>,
    ) -> Result<thread::JoinHandle<()>, FerriaError> {

        let mut analyzer = AudioAnalyzer::new(fft_size, sample_rate)?;

        let mut sample_buffer: Vec<f32> = Vec::with_capacity(fft_size);

        let handle = thread::spawn(move || {

            loop {
                //サンプルバッファがいっぱいになるまでサンプルを受信
                while sample_buffer.len() < fft_size {

                    match sample_rx.recv_timeout(Duration::from_millis(100)) {

                        Ok(sample) => sample_buffer.push(sample),

                        //タイムアウトしてもデータが来ていないだけなので続行させる
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                        },

                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            eprintln!("Spectrum data receiver disconnected. Analyzer thread exiting.");
                            return;
                        }
                     }
                }
            

                //バッファ溜まったらFFT続行
                match analyzer.analyze(&sample_buffer[..fft_size]) {

                    Ok(spectrum_data) => {

                        if let Err(_) = spectrum_tx.send(spectrum_data) {
                            eprintln!("Spectrum data receiver disconnected. Analyzer thread exiting.");
                            break;
                        }
                    },

                    Err(e) => {
                        //エラーが発生してもスレッドは続行?微妙かも
                        eprintln!("Error during FFT analysis: {}", e);
                    }
                }

                sample_buffer.clear();

            }

        });

        Ok(handle)
    
    }

}