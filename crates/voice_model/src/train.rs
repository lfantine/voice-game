//! Training glue for burn's `Learner`.
//! The training traits must be implemented here, in the crate defining `VoiceModel` (orphan rule).

use burn::data::dataloader::batcher::Batcher;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::train::{ClassificationOutput, InferenceStep, TrainOutput, TrainStep};

use crate::features::{N_FRAMES, N_MELS};
use crate::model::VoiceModel;

/// One example : features from `FeatureExtractor::extract` and its class id in `LABELS`
#[derive(Clone, Debug)]
pub struct VoiceItem {
    pub features: Vec<f32>,
    pub class: usize,
}

#[derive(Clone, Debug)]
pub struct VoiceBatch<B: Backend> {
    /// [batch, N_FRAMES, N_MELS]
    pub features: Tensor<B, 3>,
    /// [batch]
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Clone, Debug, Default)]
pub struct VoiceBatcher;

impl<B: Backend> Batcher<B, VoiceItem, VoiceBatch<B>> for VoiceBatcher {
    fn batch(&self, items: Vec<VoiceItem>, device: &B::Device) -> VoiceBatch<B> {
        let batch_size = items.len();
        let features: Vec<f32> = items.iter().flat_map(|item| item.features.iter().copied()).collect();
        let targets: Vec<i64> = items.iter().map(|item| item.class as i64).collect();

        VoiceBatch {
            features: Tensor::from_data(
                TensorData::new(features, [batch_size, N_FRAMES, N_MELS]),
                device,
            ),
            targets: Tensor::from_data(TensorData::new(targets, [batch_size]), device),
        }
    }
}

impl<B: Backend> VoiceModel<B> {
    pub fn forward_classification(&self, batch: VoiceBatch<B>) -> ClassificationOutput<B> {
        let output = self.forward(batch.features);
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), batch.targets.clone());

        ClassificationOutput::new(loss, output, batch.targets)
    }
}

impl<B: AutodiffBackend> TrainStep for VoiceModel<B> {
    type Input = VoiceBatch<B>;
    type Output = ClassificationOutput<B>;

    fn step(&self, batch: VoiceBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(batch);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> InferenceStep for VoiceModel<B> {
    type Input = VoiceBatch<B>;
    type Output = ClassificationOutput<B>;

    fn step(&self, batch: VoiceBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(batch)
    }
}
