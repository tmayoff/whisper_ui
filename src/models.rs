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
        WhisperModel::Tiny => return "tiny".to_string(),
        WhisperModel::Small => return "small".to_string(),
        WhisperModel::Base => return "base".to_string(),
        WhisperModel::Large => return "large-v2".to_string(),
    }
}

pub fn download_model(model: WhisperModel) -> Result<PathBuf> {
    println!("Dowloading model {:?}", model);
    let api = Api::new().unwrap();
    let repo = Repo::new("ggerganov".to_string(), hf_hub::RepoType::Model);
    let repo = api.repo(repo);

    let model_filename = get_model_filenaame(model);
    let m = repo.get(&model_filename)?;
    Ok(m)
}
