use crate::{
    audio,
    models::{download_model, WhisperModel},
    Message,
};
use anyhow::Result;
use iced::{
    subscription,
    widget::{text, Column},
    Element, Subscription,
};
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters};

pub struct Transcription {
    file: PathBuf,
    state: State,
}

enum State {
    Idle,
    Ready(PathBuf),
    Transcribing,
    Finished(String),
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
            State::Ready(_) => content = content.push(text("Starting transcription")),
            State::Transcribing => content = content.push(text("Transcribing")),
            State::Finished(_) => content = content.push(text("Finished transcribing")),
        }

        content.into()
    }

    pub fn process(&mut self) {
        self.state = State::Idle
    }

    pub fn subscription(&self) -> Subscription<i32> {
        match self.state {
            State::Transcribing => {
                subscription::unfold(0, State::Ready(self.file.clone()), move |state| {
                    process_async(state, WhisperModel::Small)
                })
            }
            _ => Subscription::none(),
        }
    }
}

async fn process_async(state: State, model: WhisperModel) -> (i32, State) {
    match state {
        State::Ready(f) => {
            let p = process(model, &f).await.expect("Failed to transcribe");

            (0, State::Finished(p))
        }
        State::Transcribing => todo!(),
        State::Finished(_) => iced::futures::future::pending().await,
        State::Idle => todo!(),
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
