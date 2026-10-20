//! Read the recorded examples and turn them into training items

use std::error::Error;
use std::path::{Path, PathBuf};

use tracing::warn;
use voice_model::{FeatureExtractor, LABELS, SAMPLE_RATE, VoiceItem, WINDOW_LEN};

/// Share of the recordings of each label kept for the validation
const VALID_RATIO: f32 = 0.2;
/// Time shifts (in samples) applied to each training recording, to get more examples
const SHIFTS: [isize; 5] = [-3200, -1600, 0, 1600, 3200];

/// The samples of every recording, for each label of `LABELS`
pub type Recordings = Vec<Vec<Vec<f32>>>;

/// Read `data_dir/<label>/*.wav` for each label
pub fn load(data_dir: &Path) -> Result<Recordings, Box<dyn Error>> {
    let mut recordings = Vec::new();
    for label in LABELS {
        let files = wav_files(&data_dir.join(label))?;
        if files.is_empty() {
            warn!("No example for '{label}', record some from the Record page of the app");
        }
        recordings.push(files.iter().map(|file| read_wav(file)).collect::<Result<_, _>>()?);
    }
    Ok(recordings)
}

/// Randomly keep `VALID_RATIO` of the recordings of each label for the validation, the others
/// are the training set (with time shifts to get more items). The same `seed` gives the same split.
pub fn split(recordings: &Recordings, seed: u64) -> Result<(Vec<VoiceItem>, Vec<VoiceItem>), Box<dyn Error>> {
    let extractor = FeatureExtractor::new();
    let mut rng = SplitMix64(seed);
    let mut train = Vec::new();
    let mut valid = Vec::new();

    for (class, samples) in recordings.iter().enumerate() {
        let mut order: Vec<usize> = (0..samples.len()).collect();
        rng.shuffle(&mut order);

        // keep at least one recording for training
        let n_valid = if samples.len() > 1 {
            ((samples.len() as f32 * VALID_RATIO).round() as usize).clamp(1, samples.len() - 1)
        } else {
            0
        };
        for (rank, &i) in order.iter().enumerate() {
            if rank < n_valid {
                let features = extractor.extract(&samples[i]);
                valid.push(VoiceItem { features, class });
            } else {
                for shift in SHIFTS {
                    let features = extractor.extract(&shifted(&samples[i], shift));
                    train.push(VoiceItem { features, class });
                }
            }
        }
    }

    if train.is_empty() || valid.is_empty() {
        return Err("not enough examples, record at least 2 per label from the Record page of the app".into());
    }
    Ok((train, valid))
}

/// Small deterministic random generator, enough to shuffle the recordings
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Fisher-Yates shuffle
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, (self.next() % (i as u64 + 1)) as usize);
        }
    }
}

/// The `.wav` files of `dir` sorted by name, empty if `dir` doesn't exist
fn wav_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "wav") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Samples of a mono wav file recorded at `SAMPLE_RATE`, as `f32` in [-1, 1]
fn read_wav(path: &Path) -> Result<Vec<f32>, Box<dyn Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    if spec.channels != 1 || spec.sample_rate != SAMPLE_RATE {
        return Err(format!("{path:?} must be mono at {SAMPLE_RATE} Hz").into());
    }

    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|s| s as f32 / scale))
                .collect::<Result<_, _>>()?
        }
    };
    Ok(samples)
}

/// `samples` delayed by `shift` (advanced if negative), padded with silence to `WINDOW_LEN`
fn shifted(samples: &[f32], shift: isize) -> Vec<f32> {
    (0..WINDOW_LEN as isize)
        .map(|i| {
            let source = i - shift;
            if source >= 0 { samples.get(source as usize).copied().unwrap_or(0.0) } else { 0.0 }
        })
        .collect()
}
