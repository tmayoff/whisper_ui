mod audio;
mod models;
mod whisper;

use iced::{
    widget::{button, column, horizontal_space, row, text, text_editor},
    Command,
};
use models::WhisperModel;
use rfd::FileDialog;
use std::path::PathBuf;
use whisper::State;

fn main() -> iced::Result {
    iced::program("whisper ui", App::update, App::view)
        .theme(App::theme)
        .run()
}

struct App {
    file_to_process: Option<PathBuf>,
    selected_model: WhisperModel,
    error: Option<String>,
    transcription: Option<whisper::Transcription>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    fn new() -> Self {
        App {
            file_to_process: None,
            selected_model: WhisperModel::Base,
            error: None,
            transcription: None,
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::CatppuccinMocha
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        match event {
            Message::SelectFile => {
                let file = FileDialog::new().pick_file();
                match file {
                    Some(f) => {
                        self.transcription = Some(whisper::Transcription::new(&f));
                        self.file_to_process = Some(f);
                    }
                    None => {
                        println!("File selection aborted");
                    }
                }
            }
            Message::Process => match &mut self.transcription {
                Some(t) => {
                    return t.process();
                }
                None => {
                    println!("no file to transcribe");
                }
            },
            Message::SelectModel(m) => println!("Selected model {:?}", m),
            Message::Processed(s) => {
                if let Some(t) = &mut self.transcription {
                    t.finished(&s);
                }
            }
            Message::Error(e) => self.error = Some(e),
            Message::EditorUpdate(e) => {
                if let Some(t) = &mut self.transcription {
                    t.update(e);
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let header = row![text("whisper ui").size(30), horizontal_space()].spacing(10);

        let controls = match &self.file_to_process {
            Some(file) => column![
                text(file.to_str().unwrap()).size(20),
                button("Process File").on_press(Message::Process)
            ]
            .align_items(iced::Alignment::Center)
            .spacing(10)
            .padding(10),
            None => column![button("Select File").on_press(Message::SelectFile)],
        };

        let content = row![controls].push_maybe(
            self.transcription
                .as_ref()
                .map(whisper::Transcription::view),
        );

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
    Error(String),
    SelectFile,
    Process,
    Processed(String),
    EditorUpdate(text_editor::Action),
}
