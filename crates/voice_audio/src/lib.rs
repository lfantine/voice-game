//! Microphone capture : default input device -> mono `f32` samples at a chosen sample rate

use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc::{self, Receiver, Sender};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ErrorKind, FromSample, SampleFormat, SizedSample, StreamConfig};

#[derive(Debug)]
pub enum AudioError {
    /// No input device available on the default host
    NoInputDevice,
    /// The device uses a sample format we don't handle
    UnsupportedFormat(SampleFormat),
    /// Error reported by cpal (opening or running the stream)
    Stream(cpal::Error),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::NoInputDevice => write!(f, "no input device found"),
            AudioError::UnsupportedFormat(format) => {
                write!(f, "unsupported sample format '{format}'")
            }
            AudioError::Stream(e) => write!(f, "audio stream error: {e}"),
        }
    }
}

impl std::error::Error for AudioError {}

impl From<cpal::Error> for AudioError {
    fn from(e: cpal::Error) -> Self {
        AudioError::Stream(e)
    }
}

/// What the audio callback sends to the `Microphone`
type Block = Result<Vec<f32>, cpal::Error>;

/// An open input stream, samples are pulled with `next_chunk`
pub struct Microphone {
    // the stream stops when dropped, so we keep it alive with the microphone
    _stream: cpal::Stream,
    blocks: Receiver<Block>,
    resampler: Resampler,
    /// Samples already resampled, waiting to be returned
    buffer: VecDeque<f32>,
}

impl Microphone {
    /// Open the default input device, samples will be returned mono at `sample_rate`
    pub fn open(sample_rate: u32) -> Result<Self, AudioError> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;
        let supported = device.default_input_config()?;
        let format = supported.sample_format();
        let config: StreamConfig = supported.into();

        let (tx, blocks) = mpsc::channel();
        let stream = match format {
            SampleFormat::I8 => build_stream::<i8>(&device, &config, tx)?,
            SampleFormat::I16 => build_stream::<i16>(&device, &config, tx)?,
            SampleFormat::I32 => build_stream::<i32>(&device, &config, tx)?,
            SampleFormat::U8 => build_stream::<u8>(&device, &config, tx)?,
            SampleFormat::U16 => build_stream::<u16>(&device, &config, tx)?,
            SampleFormat::F32 => build_stream::<f32>(&device, &config, tx)?,
            SampleFormat::F64 => build_stream::<f64>(&device, &config, tx)?,
            other => return Err(AudioError::UnsupportedFormat(other)),
        };
        stream.play()?;

        Ok(Self {
            _stream: stream,
            blocks,
            resampler: Resampler::new(config.sample_rate, sample_rate),
            buffer: VecDeque::new(),
        })
    }

    /// Block until `len` samples are available and return them
    pub fn next_chunk(&mut self, len: usize) -> Result<Vec<f32>, AudioError> {
        while self.buffer.len() < len {
            // the sender lives in the stream callback, owned by `self`, so it can't be dropped
            let block = self.blocks.recv().expect("audio stream callback dropped")?;
            self.resampler.push(&block, &mut self.buffer);
        }
        Ok(self.buffer.drain(..len).collect())
    }

    /// Drop every sample captured so far, the next chunk starts now
    pub fn flush(&mut self) {
        while self.blocks.try_recv().is_ok() {}
        self.buffer.clear();
    }
}

/// Build an input stream sending mono `f32` blocks into `tx`
fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    tx: Sender<Block>,
) -> Result<cpal::Stream, AudioError>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    let err_tx = tx.clone();

    let on_data = move |data: &[T], _: &cpal::InputCallbackInfo| {
        let mono = data
            .chunks(channels)
            .map(|frame| frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32)
            .collect();
        let _ = tx.send(Ok(mono));
    };
    let on_error = move |err: cpal::Error| match err.kind() {
        // lost a few samples, the stream keeps going
        ErrorKind::Xrun | ErrorKind::RealtimeDenied => {}
        _ => {
            let _ = err_tx.send(Err(err));
        }
    };

    Ok(device.build_input_stream(config.clone(), on_data, on_error, None)?)
}

/// Linear interpolation resampler, keeps its state between blocks
struct Resampler {
    /// Input samples consumed per output sample
    step: f64,
    /// Position of the next output sample in `pending`
    pos: f64,
    /// Input samples not fully consumed yet
    pending: Vec<f32>,
}

impl Resampler {
    fn new(from: u32, to: u32) -> Self {
        Self {
            step: from as f64 / to as f64,
            pos: 0.0,
            pending: Vec::new(),
        }
    }

    fn push(&mut self, input: &[f32], out: &mut VecDeque<f32>) {
        self.pending.extend_from_slice(input);

        while self.pos + 1.0 < self.pending.len() as f64 {
            let i = self.pos as usize;
            let frac = (self.pos - i as f64) as f32;
            out.push_back(self.pending[i] + (self.pending[i + 1] - self.pending[i]) * frac);
            self.pos += self.step;
        }

        let consumed = (self.pos as usize).min(self.pending.len());
        self.pending.drain(..consumed);
        self.pos -= consumed as f64;
    }
}
