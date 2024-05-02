use anyhow::Result;
use hf_hub::{api::sync::Api, Repo};
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Copy)]
pub enum WhisperModel {
    Tiny,
    #[default]
    Small,
    Base,
    Large,
}

impl WhisperModel {
    pub const ALL: [WhisperModel; 4] = [
        WhisperModel::Tiny,
        WhisperModel::Small,
        WhisperModel::Base,
        WhisperModel::Large,
    ];
}

impl std::fmt::Display for WhisperModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WhisperModel::Tiny => "Tiny",
                WhisperModel::Small => "Small",
                WhisperModel::Base => "Base",
                WhisperModel::Large => "Large",
            }
        )
    }
}

fn get_model_filenaame(model: WhisperModel) -> String {
    match model {
        WhisperModel::Tiny => return "ggml-tiny.bin".to_string(),
        WhisperModel::Small => return "ggml-small.bin".to_string(),
        WhisperModel::Base => return "ggml-base.bin".to_string(),
        WhisperModel::Large => return "ggml-large-v2.bin".to_string(),
    }
}

pub fn download_model(model: WhisperModel) -> Result<PathBuf> {
    println!("Dowloading model {:?}", model);
    let api = Api::new().unwrap();
    let repo = Repo::new("ggerganov/whisper.cpp".to_string(), hf_hub::RepoType::Model);
    let repo = api.repo(repo);

    let model_filename = get_model_filenaame(model);
    let m = repo.get(&model_filename)?;
    Ok(m)
}
