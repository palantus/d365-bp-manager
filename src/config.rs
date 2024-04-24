use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub modelpath: String,
    pub models: Vec<Model>,
}

#[derive(Deserialize, Clone, Default)]
pub struct Model {
    pub name: String,
    pub alias: String,
}

pub fn read_config() -> Result<(Config, Model), String> {
    let args: Vec<String> = std::env::args().collect();
    // println!("{0:?}", args);
    if args.len() < 2 {
        return Err("No model argument specified".to_owned());
    }
    let arg_name = args[1].clone();
    let toml = fs::read_to_string("./config.toml").unwrap();
    let config: Config = toml::from_str(&toml).unwrap();

    let model = match config
        .models
        .iter()
        .find(|m| m.name == arg_name || m.alias == arg_name)
    {
        Some(m) => m.clone(),
        None => return Err("Could not find model in config.toml".to_owned()),
    };

    let models_path = Path::new(&config.modelpath);
    if !models_path.exists() {
        return Err("Base model path in config doesn't exist".to_owned());
    }
    let model_path = models_path.join(&model.name);
    if !model_path.exists() {
        return Err("Model in config doesn't exist".to_owned());
    }

    let bp_file_path = model_path.join("BPCheck.xml");
    if !bp_file_path.exists() {
        return Err("BPCheck.xml doesn't exist in model".to_owned());
    }
    Ok((config, model))
}
