use crate::{
    audio,
    models::{download_model, WhisperModel},
    Message,
};
use anyhow::Result;
use iced::{
    widget::{text, Column},
    Command, Element,
};
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters};

#[derive(Clone)]
pub struct Transcription {
    file: PathBuf,
    pub state: State,
}

#[derive(Clone)]
pub enum State {
    Idle,
    Finished(String),
    Transcribing,
}

impl Transcription {
    pub fn new(file: &Path) -> Self {
        Self {
            file: file.to_path_buf(),
            state: State::Idle,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut content = Column::new();
        match self.state {
            State::Idle => content = content.push(text("Waiting to transcribe")),
            State::Finished(_) => content = content.push(text("Finished transcribing")),
            State::Transcribing => content = content.push(text("Transcribing")),
        }

        content.into()
    }

    pub fn process(&mut self) -> Command<Message> {
        self.state = State::Transcribing;
        let file = self.file.clone();
        Command::perform(
            async move { process(WhisperModel::Small, &file).await },
            |res| match res {
                Ok(s) => Message::Processed(s),
                Err(e) => Message::Error(e.to_string()),
            },
        )
    }
}

async fn process(model: WhisperModel, file: &Path) -> Result<String> {
    println!("Processing: {:?}...", file);
    let model_file = download_model(model).expect("Failed to get model file");

    let params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });

    let audio = audio::read_audio(file)?;

    let ctx = WhisperContext::new_with_params(
        model_file.to_str().unwrap(),
        WhisperContextParameters::default(),
    )
    .expect("failed to get whisper context");

    let mut state = ctx.create_state()?;
    state.full(params, &audio[..])?;

    let mut text = String::new();
    let num_segments = state.full_n_segments()?;
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i)?;
        text.push_str(&segment);
        text.push('\n');
    }

    Ok(text)
}
