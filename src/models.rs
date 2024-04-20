use anyhow::Result;
use hf_hub::{api::sync::Api, Repo};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum WhisperModel {
    Tiny,
    Small,
    Base,
    Large,
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
