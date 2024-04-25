use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub modelpath: String,
    pub models: Vec<String>,
}

pub fn read_config() -> Result<Config, String> {
    let toml = match fs::read_to_string("./config.toml") {
        Ok(toml) => toml,
        Err(_e) => return Err("Config file missing".to_owned()),
    };
    let config: Config = match toml::from_str(&toml) {
        Ok(c) => c,
        Err(_e) => return Err("Config file could not be parsed".to_owned()),
    };

    let models_path = Path::new(&config.modelpath);
    if !models_path.exists() {
        return Err("Base model path in config doesn't exist".to_owned());
    }
    Ok(config)
}
