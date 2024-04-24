use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub modelpath: String,
    pub models: Vec<Model>,
}

#[derive(Deserialize, Clone)]
pub struct Model {
    pub name: String,
    pub alias: String,
}

pub fn read_config() -> (Config, Model) {
    let args: Vec<String> = std::env::args().collect();
    // println!("{0:?}", args);
    if args.len() < 2 {
        panic!("No model argument specified")
    }
    let arg_name = args[1].clone();
    let toml = fs::read_to_string("./config.toml").unwrap();
    let config: Config = toml::from_str(&toml).unwrap();

    let model = config
        .models
        .iter()
        .find(|m| m.name == arg_name || m.alias == arg_name)
        .expect("Could not find model in config.toml")
        .clone();

    let models_path = Path::new(&config.modelpath);
    if !models_path.exists() {
        panic!("Base model path in config doesn't exist");
    }
    let model_path = models_path.join(&model.name);
    if !model_path.exists() {
        panic!("Model in config doesn't exist");
    }

    let bp_file_path = model_path.join("BPCheck.xml");
    if !bp_file_path.exists() {
        panic!("BPCheck.xml doesn't exist in model");
    }
    (config, model)
}
