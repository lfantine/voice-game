//! The network : log-mel spectrogram -> class logits

use std::path::Path;

use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig, MaxPool2d, MaxPool2dConfig};
use burn::nn::{BatchNorm, BatchNormConfig, Dropout, DropoutConfig, Linear, LinearConfig, PaddingConfig2d, Relu};
use burn::prelude::*;
use burn::record::CompactRecorder;

use crate::NUM_CLASSES;
use crate::inference::ModelError;

/// Small CNN over the spectrogram seen as a 1 channel image
#[derive(Module, Debug)]
pub struct VoiceModel<B: Backend> {
    blocks: Vec<ConvBlock<B>>,
    global_pool: AdaptiveAvgPool2d,
    dropout: Dropout,
    hidden: Linear<B>,
    output: Linear<B>,
    activation: Relu,
}

#[derive(Config, Debug)]
pub struct VoiceModelConfig {
    #[config(default = "[16, 32, 64]")]
    channels: [usize; 3],
    #[config(default = 64)]
    hidden_size: usize,
    #[config(default = 0.3)]
    dropout: f64,
}

impl VoiceModelConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> VoiceModel<B> {
        let mut blocks = Vec::new();
        let mut in_channels = 1;
        for &out_channels in &self.channels {
            blocks.push(ConvBlock::new(in_channels, out_channels, device));
            in_channels = out_channels;
        }

        VoiceModel {
            blocks,
            global_pool: AdaptiveAvgPool2dConfig::new([1, 1]).init(),
            dropout: DropoutConfig::new(self.dropout).init(),
            hidden: LinearConfig::new(in_channels, self.hidden_size).init(device),
            output: LinearConfig::new(self.hidden_size, NUM_CLASSES).init(device),
            activation: Relu::new(),
        }
    }
}

impl<B: Backend> VoiceModel<B> {
    /// `features` : [batch, N_FRAMES, N_MELS] -> logits : [batch, NUM_CLASSES]
    pub fn forward(&self, features: Tensor<B, 3>) -> Tensor<B, 2> {
        let mut x = features.unsqueeze_dim::<4>(1);
        for block in &self.blocks {
            x = block.forward(x);
        }

        let x = self.global_pool.forward(x).flatten::<2>(1, 3);
        let x = self.dropout.forward(x);
        let x = self.activation.forward(self.hidden.forward(x));
        self.output.forward(x)
    }

    /// Save the weights to `path` (the extension is set to `.mpk`)
    pub fn save(self, path: &Path) -> Result<(), ModelError> {
        Ok(self.save_file(path, &CompactRecorder::new())?)
    }

    /// Load the weights saved by `save` into this model
    pub fn load(self, path: &Path, device: &B::Device) -> Result<Self, ModelError> {
        Ok(self.load_file(path, &CompactRecorder::new(), device)?)
    }
}

/// conv 3x3 -> batch norm -> relu -> max pool 2x2
#[derive(Module, Debug)]
struct ConvBlock<B: Backend> {
    conv: Conv2d<B>,
    norm: BatchNorm<B>,
    pool: MaxPool2d,
    activation: Relu,
}

impl<B: Backend> ConvBlock<B> {
    fn new(in_channels: usize, out_channels: usize, device: &B::Device) -> Self {
        Self {
            conv: Conv2dConfig::new([in_channels, out_channels], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .init(device),
            norm: BatchNormConfig::new(out_channels).init(device),
            pool: MaxPool2dConfig::new([2, 2]).with_strides([2, 2]).init(),
            activation: Relu::new(),
        }
    }

    fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.norm.forward(self.conv.forward(x));
        self.pool.forward(self.activation.forward(x))
    }
}
