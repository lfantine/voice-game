//! Samples -> normalized log-mel spectrogram

use std::f32::consts::PI;
use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::{SAMPLE_RATE, WINDOW_LEN};

/// 25 ms analysis frames
const FRAME_LEN: usize = 400;
/// 10 ms between two frames
const HOP_LEN: usize = 160;
const N_FFT: usize = 512;
const N_BINS: usize = N_FFT / 2 + 1;
const MIN_FREQ: f32 = 20.0;

/// Number of mel bands per frame
pub const N_MELS: usize = 40;
/// Number of frames in a window of `WINDOW_LEN` samples
pub const N_FRAMES: usize = 1 + (WINDOW_LEN - FRAME_LEN) / HOP_LEN;

/// Compute the model input from a window of samples
pub struct FeatureExtractor {
    fft: Arc<dyn Fft<f32>>,
    /// Hann window applied to each frame
    window: Vec<f32>,
    /// One triangular filter per mel band, `N_MELS` rows of `N_BINS` weights
    filters: Vec<f32>,
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureExtractor {
    pub fn new() -> Self {
        let window = (0..FRAME_LEN)
            .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / FRAME_LEN as f32).cos())
            .collect();

        Self {
            fft: FftPlanner::new().plan_fft_forward(N_FFT),
            window,
            filters: mel_filters(),
        }
    }

    /// Log-mel spectrogram of `samples`, `N_FRAMES * N_MELS` values (frame major),
    /// normalized to zero mean and unit variance.
    /// `samples` is truncated or padded with zeros to `WINDOW_LEN`.
    pub fn extract(&self, samples: &[f32]) -> Vec<f32> {
        let mut padded = samples[..samples.len().min(WINDOW_LEN)].to_vec();
        padded.resize(WINDOW_LEN, 0.0);

        let mut features = Vec::with_capacity(N_FRAMES * N_MELS);
        let mut buffer = vec![Complex::default(); N_FFT];
        let mut power = [0.0f32; N_BINS];

        for frame in 0..N_FRAMES {
            let start = frame * HOP_LEN;
            for (i, value) in buffer.iter_mut().enumerate() {
                let sample = if i < FRAME_LEN { padded[start + i] * self.window[i] } else { 0.0 };
                *value = Complex::new(sample, 0.0);
            }
            self.fft.process(&mut buffer);

            for (p, value) in power.iter_mut().zip(&buffer) {
                *p = value.norm_sqr();
            }
            for filter in self.filters.chunks(N_BINS) {
                let energy: f32 = filter.iter().zip(&power).map(|(w, p)| w * p).sum();
                features.push((energy + 1e-6).ln());
            }
        }

        normalize(&mut features);
        features
    }
}

fn normalize(values: &mut [f32]) {
    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    let std = var.sqrt().max(1e-5);
    for v in values {
        *v = (*v - mean) / std;
    }
}

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10f32.powf(mel / 2595.0) - 1.0)
}

/// Triangular filters, evenly spaced on the mel scale between `MIN_FREQ` and Nyquist
fn mel_filters() -> Vec<f32> {
    let min_mel = hz_to_mel(MIN_FREQ);
    let max_mel = hz_to_mel(SAMPLE_RATE as f32 / 2.0);
    // N_MELS filters need N_MELS + 2 edges
    let edges: Vec<f32> = (0..N_MELS + 2)
        .map(|i| mel_to_hz(min_mel + (max_mel - min_mel) * i as f32 / (N_MELS + 1) as f32))
        .collect();

    let mut filters = vec![0.0; N_MELS * N_BINS];
    for (band, edge) in edges.windows(3).enumerate() {
        let (left, center, right) = (edge[0], edge[1], edge[2]);
        for bin in 0..N_BINS {
            let freq = bin as f32 * SAMPLE_RATE as f32 / N_FFT as f32;
            let weight = if freq > left && freq <= center {
                (freq - left) / (center - left)
            } else if freq > center && freq < right {
                (right - freq) / (right - center)
            } else {
                0.0
            };
            filters[band * N_BINS + bin] = weight;
        }
    }
    filters
}
