#![allow(non_snake_case)]

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::config::Config;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Diagnostics {
    pub Items: Items,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Items {
    pub Diagnostic: Vec<Diagnostic>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Diagnostic {
    pub DiagnosticType: String,
    pub Severity: String,
    #[serde[default = "String::new"]]
    #[serde(skip_serializing)]
    pub ElementType: String,
    pub Path: String,
    pub Moniker: String,
    #[serde[default = "String::new"]]
    #[serde(skip_serializing)]
    pub Message: String,
    #[serde[default = "String::new"]]
    pub Justification: String,
}

pub fn read_diagnostics(config: &Config, model: &String) -> Result<Vec<Diagnostic>, String> {
    let modelsPath = Path::new(&config.modelpath);
    if !modelsPath.exists() {
        return Err("Base model path in config doesn't exist".to_owned());
    }
    let modelPath = modelsPath.join(&model);
    if !modelPath.exists() {
        return Err("Model in config doesn't exist".to_owned());
    }

    let bpFilePath = modelPath.join("BPCheck.xml");
    if !bpFilePath.exists() {
        return Err("BPCheck.xml doesn't exist in model".to_owned());
    }
    let xml = fs::read_to_string(bpFilePath).unwrap();
    let diags: Diagnostics = from_str(&xml).unwrap();
    Ok(diags.Items.Diagnostic)
}
