mod audio;
mod models;

use anyhow::Result;
use iced::{
    widget::{button, column, horizontal_space, row, scrollable, text, text_editor, text_input},
    Sandbox, Settings,
};
use models::WhisperModel;
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters};

use crate::models::download_model;

fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App {
    file_to_process: Option<PathBuf>,
    selected_model: WhisperModel,
    error: Option<String>,
    transcription: Option<text_editor::Content>,
}

fn whisper_process(model: WhisperModel, file: &Path) -> Result<String> {
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

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {
            file_to_process: None,
            selected_model: WhisperModel::Base,
            error: None,
            transcription: None,
        }
    }

    fn title(&self) -> String {
        format!("WhisperUI")
    }

    fn update(&mut self, event: Self::Message) {
        match event {
            Message::SelectFile => {
                let file = FileDialog::new().pick_file();
                match file {
                    Some(f) => {
                        println!("File Selected: {:?}", f);
                        self.file_to_process = Some(f);
                    }
                    None => println!("File selection aborted"),
                }
            }
            Message::Process(file) => {
                let text = whisper_process(self.selected_model, &file);
                match text {
                    Ok(t) => self.transcription = Some(text_editor::Content::with_text(&t)),
                    Err(e) => self.error = Some(format!("{:?}", e)),
                }
            }
            Message::SelectModel(_) => todo!(),
            Message::TranscriptionUpdate(action) => {
                if let Some(t) = &mut self.transcription {
                    t.perform(action);
                }
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let header = row![text("whisper ui").size(30), horizontal_space()].spacing(10);

        let controls = match &self.file_to_process {
            Some(file) => column![
                text(file.to_str().unwrap()).size(20),
                button("Process File").on_press(Message::Process(file.to_owned()))
            ]
            .align_items(iced::Alignment::Center)
            .spacing(10)
            .padding(10),
            None => column![button("Select File").on_press(Message::SelectFile)],
        };

        let mut content = row![controls];
        if let Some(t) = &self.transcription {
            content = content.push(text_editor(t).on_action(Message::TranscriptionUpdate));
        }

        column![header, content]
            .align_items(iced::Alignment::Center)
            .spacing(10)
            .padding(10)
            .into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    SelectModel(WhisperModel),
    SelectFile,
    TranscriptionUpdate(text_editor::Action),
    Process(PathBuf),
}
