//! Load trained weights and turn windows of samples into commands

use std::fmt;
use std::path::Path;

use burn::prelude::*;
use burn::record::RecorderError;
use burn::tensor::activation::softmax;

use crate::features::{FeatureExtractor, N_FRAMES, N_MELS};
use crate::model::{VoiceModel, VoiceModelConfig};
use crate::Command;

/// Below this RMS the window is considered silent, the model is not even run
const SILENCE_RMS: f32 = 0.005;
/// Minimum probability of the best class to accept a prediction
const MIN_CONFIDENCE: f32 = 0.7;

#[derive(Debug)]
pub enum ModelError {
    /// Weights missing, unreadable or not matching the model
    Weights(RecorderError),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::Weights(e) => write!(f, "model weights error: {e}"),
        }
    }
}

impl std::error::Error for ModelError {}

impl From<RecorderError> for ModelError {
    fn from(e: RecorderError) -> Self {
        ModelError::Weights(e)
    }
}

/// A trained model ready to make predictions
pub struct Recognizer<B: Backend> {
    model: VoiceModel<B>,
    features: FeatureExtractor,
    device: B::Device,
}

impl<B: Backend> Recognizer<B> {
    /// Load the weights saved by the training
    pub fn load(weights: &Path, device: B::Device) -> Result<Self, ModelError> {
        let model = VoiceModelConfig::new().init(&device).load(weights, &device)?;
        Ok(Self {
            model,
            features: FeatureExtractor::new(),
            device,
        })
    }

    /// The command spoken in `samples` (mono, `SAMPLE_RATE`), if any
    pub fn predict(&self, samples: &[f32]) -> Option<Command> {
        if rms(samples) < SILENCE_RMS {
            return None;
        }

        let features = TensorData::new(self.features.extract(samples), [1, N_FRAMES, N_MELS]);
        let input = Tensor::<B, 3>::from_data(features, &self.device);
        let probs = softmax(self.model.forward(input), 1)
            .into_data()
            .into_vec::<f32>()
            .ok()?;

        let (class, &confidence) = probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))?;
        if confidence < MIN_CONFIDENCE {
            return None;
        }
        Command::from_class(class)
    }
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}
