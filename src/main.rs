mod models;
mod audio;

use std::path::{Path, PathBuf};

use iced::{
    widget::{button, row, text},
    Sandbox, Settings,
};
use models::WhisperModel;
use rfd::FileDialog;
use whisper_rs::{WhisperContext, WhisperContextParameters};

use crate::models::download_model;

fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App {
    file_to_process: Option<PathBuf>,
    selected_model: WhisperModel,
}

fn whisper_process(model: WhisperModel, file: &Path) {
    let model_file = download_model(model).expect("Failed to get model file");

    let ctx = WhisperContext::new_with_params(
        model_file.to_str().unwrap(),
        WhisperContextParameters::default(),
    )
    .expect("failed to get whisper context");

    

    println!("Processing: {:?}", file);
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {
            file_to_process: None,
            selected_model: WhisperModel::Small,
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
                whisper_process(self.selected_model, &file);
            }
            Message::SelectModel(_) => todo!(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let header = row![text("whisper").size(20),].align_items(iced::Alignment::Center);

        let mut content = row![];
        content = match &self.file_to_process {
            Some(file) => {
                content.push(button("Process File").on_press(Message::Process(file.to_owned())))
            }
            None => content.push(button("Select File").on_press(Message::SelectFile)),
        };

        row![header, content].into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    SelectModel(WhisperModel),
    SelectFile,
    Process(PathBuf),
}
