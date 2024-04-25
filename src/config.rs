use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub modelpath: String,
    pub models: Vec<String>,
}

pub fn read_config() -> Result<Config, String> {
    let toml = fs::read_to_string("./config.toml").unwrap();
    let config: Config = toml::from_str(&toml).unwrap();

    let models_path = Path::new(&config.modelpath);
    if !models_path.exists() {
        return Err("Base model path in config doesn't exist".to_owned());
    }
    Ok(config)
}
