mod audio;
mod models;
mod whisper;

use iced::{
    widget::{
        button, column, combo_box, container, horizontal_rule, horizontal_space, row, text, tooltip,
    },
    Command,
};
use models::WhisperModel;
use rfd::FileDialog;
use std::path::PathBuf;

fn main() -> iced::Result {
    iced::program("whisper ui", App::update, App::view)
        .theme(App::theme)
        .run()
}

struct App {
    file_to_process: Option<PathBuf>,
    model_selection: combo_box::State<WhisperModel>,
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
            model_selection: combo_box::State::new(WhisperModel::ALL.to_vec()),
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
            Message::Process => {
                // Forwardt this event to the transcription 'widget'
                return self.update(Message::TranscriptionEvent(whisper::Event::Process(
                    self.selected_model,
                )));
            }
            Message::SelectModel(m) => self.selected_model = m,
            Message::Error(e) => self.error = Some(e),
            Message::TranscriptionEvent(m) => {
                // Forward events to the transcription 'widget'
                if let Some(t) = &mut self.transcription {
                    return t.update(m);
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let header = row![text("whisper ui").size(30), horizontal_space()]
            .spacing(10)
            .align_items(iced::Alignment::Center);

        let filename = self
            .file_to_process
            .as_ref()
            .map(|file| text(file.to_str().unwrap()).size(20));
        let controls = column![]
            .push_maybe(filename)
            .spacing(10)
            .push(
                row![
                    match &self.file_to_process {
                        Some(_) => button("Process file").on_press(Message::Process),
                        None => button("Select a file to transcribe").on_press(Message::SelectFile),
                    },
                    tooltip(
                        text("Choose whisper model to use:"),
                        container(column![
                            text("The larger the model the better the accuracy"),
                            text("but the longer it'll take."),
                            text("Base is a good default")
                        ])
                        .padding(15),
                        tooltip::Position::Bottom
                    ),
                    combo_box(
                        &self.model_selection,
                        "",
                        Some(&self.selected_model),
                        Message::SelectModel,
                    ),
                ]
                .spacing(10),
            )
            .max_width(650)
            .padding(10);

        let content = column![controls, horizontal_rule(3)]
            .push_maybe(
                self.transcription
                    .as_ref()
                    .map(whisper::Transcription::view),
            )
            .align_items(iced::Alignment::Center);

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
    TranscriptionEvent(whisper::Event),
}
