use crate::{
    audio,
    models::{download_model, WhisperModel},
};
use anyhow::Result;
use iced::{
    widget::{text, text_editor, Column},
    Element, Task,
};
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters};

pub struct Transcription {
    file: PathBuf,
    state: State,
    editor: Option<text_editor::Content>,
}

#[derive(Clone)]
pub enum State {
    Idle,
    Finished,
    Transcribing,
}

#[derive(Debug, Clone)]
pub enum Event {
    Process(WhisperModel),
    Processed(String),
    EditorUpdate(text_editor::Action),
}

impl Transcription {
    pub fn new(file: &Path) -> Self {
        Self {
            file: file.to_path_buf(),
            state: State::Idle,
            editor: None,
        }
    }

    pub fn update(&mut self, event: Event) -> Task<crate::Message> {
        match event {
            Event::EditorUpdate(a) => {
                if let Some(e) = &mut self.editor {
                    e.perform(a);
                }
            }
            Event::Process(model) => return self.process(model),
            Event::Processed(s) => {
                self.state = State::Finished;
                self.editor = Some(text_editor::Content::with_text(&s));
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<crate::Message> {
        let mut content = Column::new();
        match self.state {
            State::Idle => content = content.push(text("Waiting to transcribe")),
            State::Finished => (),
            State::Transcribing => content = content.push(text("Transcribing")),
        }

        let editor = self
            .editor
            .as_ref()
            .map(|e| text_editor(&e).on_action(|a| Self::wrap_event(Event::EditorUpdate(a))));

        content = content.push_maybe(editor);

        content.into()
    }

    pub fn process(&mut self, model: WhisperModel) -> Task<crate::Message> {
        self.state = State::Transcribing;
        let file = self.file.clone();
        Task::perform(
            async move { process(model, &file).await },
            |res| match res {
                Ok(s) => crate::Message::TranscriptionEvent(Event::Processed(s)),
                Err(e) => crate::Message::Error(e.to_string()),
            },
        )
    }

    fn wrap_event(event: Event) -> crate::Message {
        crate::Message::TranscriptionEvent(event)
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
