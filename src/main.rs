mod audio;
mod models;
mod whisper;

use async_std::println;
use iced::{
    executor,
    widget::{button, column, horizontal_space, row, text},
    Application, Command, Sandbox, Settings, Theme,
};
use models::WhisperModel;
use rfd::FileDialog;
use std::path::PathBuf;

fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App {
    file_to_process: Option<PathBuf>,
    selected_model: WhisperModel,
    error: Option<String>,
    transcription: Option<whisper::Transcription>,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            App {
                file_to_process: None,
                selected_model: WhisperModel::Base,
                error: None,
                transcription: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("WhisperUI")
    }

    fn update(&mut self, event: Self::Message) -> Command<Message> {
        match event {
            Message::SelectFile => {
                let file = FileDialog::new().pick_file();
                match file {
                    Some(f) => {
                        self.transcription = Some(whisper::Transcription::new(&f));
                        self.file_to_process = Some(f);
                    }
                    None => println!("File selection aborted"),
                }
            }
            Message::Process => match &mut self.transcription {
                Some(t) => t.process(),
                None => println!("no file to transcribe"),
            },
            Message::SelectModel(m) => println!("Selected model {:?}", m),
            Message::Processed => println!("Processed file"),
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
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

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match &self.transcription {
            Some(t) => t.subscription().map(|_| Message::Processed),
            None => iced::Subscription::none(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SelectModel(WhisperModel),
    SelectFile,
    Process,
    Processed,
}
