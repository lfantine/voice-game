//! Training of the voice model : examples are recorded from the app, `--train` fits the model

mod dataset;
mod record;

pub use record::{record_sample, sample_count};

use std::error::Error;
use std::path::PathBuf;

use burn::backend::{Autodiff, Wgpu};
use burn::data::dataloader::batcher::Batcher;
use burn::data::dataloader::DataLoaderBuilder;
use burn::data::dataset::InMemDataset;
use burn::optim::AdamConfig;
use burn::record::CompactRecorder;
use burn::train::metric::{AccuracyMetric, LossMetric};
use burn::train::{Learner, SupervisedTraining};
use tracing::info;
use voice_model::{NUM_CLASSES, VoiceBatcher, VoiceItem, VoiceModel, VoiceModelConfig};

use crate::voice::weights_path;

type TrainBackend = Autodiff<Wgpu>;

const NUM_EPOCHS: usize = 30;
const BATCH_SIZE: usize = 32;
const LEARNING_RATE: f64 = 1e-3;
const SEED: u64 = 42;
/// The training is accepted when this share of the validation set is well predicted
const TARGET_ACCURACY: f32 = 0.9;
/// Number of trainings (each one with a new random split) before keeping the best one
const MAX_ATTEMPTS: u64 = 3;

/// Folder holding the recorded examples, one sub folder per label
fn data_dir() -> PathBuf {
    std::env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string()).into()
}

/// Train the model on the examples of `data_dir()` and save the weights to `weights_path()`.
/// 80 % of the recordings train the model and 20 % validate it. If the validation accuracy
/// is under `TARGET_ACCURACY` the training is run again on a new split, up to `MAX_ATTEMPTS`
/// times, and the best weights are kept.
pub fn run() -> Result<(), Box<dyn Error>> {
    let recordings = dataset::load(&data_dir())?;
    let weights = weights_path();
    let mut best = None::<f32>;

    for attempt in 1..=MAX_ATTEMPTS {
        println!("--- Attempt {attempt}/{MAX_ATTEMPTS} ---");
        let (train, valid) = dataset::split(&recordings, SEED + attempt)?;
        info!("{} training items, {} validation items", train.len(), valid.len());

        let model = fit(train, valid.clone(), attempt)?;
        let accuracy = accuracy(&model, &valid);
        println!("Validation accuracy of attempt {attempt}: {:.1} %", accuracy * 100.0);

        if best.is_none_or(|best| accuracy > best) {
            model.save(&weights)?;
            println!("Weights saved to {weights:?}");
            best = Some(accuracy);
        }
        if accuracy >= TARGET_ACCURACY {
            return Ok(());
        }
        if attempt < MAX_ATTEMPTS {
            println!("Under {:.0} %, training again on a new split", TARGET_ACCURACY * 100.0);
        }
    }
    println!(
        "Target accuracy not reached, best is {:.1} %. Record more varied examples to improve it.",
        best.unwrap_or(0.0) * 100.0
    );
    Ok(())
}

/// One training of a new model, returns it without the autodiff wrapper
fn fit(
    train: Vec<VoiceItem>,
    valid: Vec<VoiceItem>,
    attempt: u64,
) -> Result<VoiceModel<Wgpu>, Box<dyn Error>> {
    // logs and checkpoints of the learner, next to the weights
    let training_dir = weights_path().with_file_name("training").join(format!("attempt-{attempt}"));

    let train_loader = DataLoaderBuilder::<TrainBackend, _, _>::new(VoiceBatcher)
        .batch_size(BATCH_SIZE)
        .shuffle(SEED + attempt)
        .build(InMemDataset::new(train));
    let valid_loader = DataLoaderBuilder::<Wgpu, _, _>::new(VoiceBatcher)
        .batch_size(BATCH_SIZE)
        .build(InMemDataset::new(valid));

    let model = VoiceModelConfig::new().init::<TrainBackend>(&Default::default());
    let learner = Learner::new(model, AdamConfig::new().init(), LEARNING_RATE);

    let result = SupervisedTraining::new(&training_dir, train_loader, valid_loader)
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .num_epochs(NUM_EPOCHS)
        .summary()
        .launch(learner);

    Ok(result.model)
}

/// Share of `items` whose class is well predicted by `model`
fn accuracy(model: &VoiceModel<Wgpu>, items: &[VoiceItem]) -> f32 {
    let device = Default::default();
    let mut correct = 0;
    for chunk in items.chunks(BATCH_SIZE) {
        let batch = Batcher::<Wgpu, _, _>::batch(&VoiceBatcher, chunk.to_vec(), &device);
        let scores = model
            .forward(batch.features)
            .into_data()
            .into_vec::<f32>()
            .expect("the model outputs floats");
        for (item, scores) in chunk.iter().zip(scores.chunks(NUM_CLASSES)) {
            let predicted = scores
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(class, _)| class);
            correct += (predicted == Some(item.class)) as usize;
        }
    }
    correct as f32 / items.len() as f32
}
