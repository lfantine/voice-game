//! Record page : pick a label and record 1 s examples of it for `--train`

use iced::widget::{button, center, column, row, text};
use iced::{Center, Element};
use voice_model::{LABELS, NUM_CLASSES};

use super::window::AppMessage;
use crate::train;

#[derive(Debug)]
pub struct RecordState {
    /// Label being recorded
    pub label: &'static str,
    /// A recording is running, the microphone is open
    pub recording: bool,
    /// Number of examples on disk, for each label of `LABELS`
    counts: [usize; NUM_CLASSES],
    status: String,
}

impl Default for RecordState {
    fn default() -> Self {
        let mut state = Self {
            label: LABELS[0],
            recording: false,
            counts: [0; NUM_CLASSES],
            status: String::new(),
        };
        state.refresh();
        state
    }
}

impl RecordState {
    /// Count the examples on disk again
    pub fn refresh(&mut self) {
        for (count, label) in self.counts.iter_mut().zip(LABELS) {
            *count = train::sample_count(label);
        }
    }

    /// Record clicked, the microphone is not open yet
    pub fn start(&mut self) {
        self.recording = true;
        self.status = "Get ready...".to_string();
    }

    /// The microphone is open
    pub fn listening(&mut self) {
        self.status = format!("Recording... say '{}' now !", self.label);
    }

    /// Store the outcome of a recording
    pub fn finish(&mut self, result: Result<String, String>) {
        self.recording = false;
        self.status = match result {
            Ok(file) => format!("Saved {file}"),
            Err(e) => format!("Error: {e}"),
        };
        self.refresh();
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        let labels = row(LABELS.iter().zip(self.counts).map(|(&label, count)| {
            let style = if label == self.label {
                button::primary
            } else {
                button::secondary
            };
            button(text(format!("{label} ({count})")))
                .style(style)
                .on_press_maybe((!self.recording).then_some(AppMessage::SelectLabel(label)))
                .into()
        }))
        .spacing(10);

        let hint = if self.label == "noise" {
            "Stay quiet or make some background noise".to_string()
        } else {
            format!("Click Record, wait for 'Recording...' then say '{}' (1 s)", self.label)
        };

        center(
            column![
                text("Record examples").size(40),
                labels,
                text(hint),
                button(text("Record").size(20))
                    .padding([10, 40])
                    .on_press_maybe((!self.recording).then_some(AppMessage::StartRecording)),
                text(&self.status),
                button(text("Back")).on_press_maybe((!self.recording).then_some(AppMessage::Quit)),
            ]
            .spacing(25)
            .align_x(Center),
        )
        .into()
    }
}
