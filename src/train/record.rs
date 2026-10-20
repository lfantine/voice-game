//! Record labelled examples from the microphone

use std::error::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use voice_audio::Microphone;
use voice_model::{LABELS, SAMPLE_RATE, WINDOW_LEN, class_of};

use super::data_dir;

/// Record 1 s of `label` from the microphone and save it in `data_dir()/<label>/`.
/// Blocks for the duration of the recording, the microphone is only open during the call.
pub fn record_sample(label: &str) -> Result<PathBuf, Box<dyn Error>> {
    if class_of(label).is_none() {
        return Err(format!("unknown label '{label}', expected one of {LABELS:?}").into());
    }
    let dir = data_dir().join(label);
    std::fs::create_dir_all(&dir)?;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut mic = Microphone::open(SAMPLE_RATE)?;
    let samples = mic.next_chunk(WINDOW_LEN)?;

    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let path = dir.join(format!("{millis}.wav"));
    let mut writer = hound::WavWriter::create(&path, spec)?;
    for sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(path)
}

/// Number of examples already recorded for `label`
pub fn sample_count(label: &str) -> usize {
    std::fs::read_dir(data_dir().join(label))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "wav"))
                .count()
        })
        .unwrap_or(0)
}
