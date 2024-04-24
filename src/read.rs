#![allow(non_snake_case)]

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::config::{Config, Model};

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

pub fn read_diagnostics(config: &Config, model: &Model) -> Vec<Diagnostic> {
    let modelsPath = Path::new(&config.modelpath);
    if !modelsPath.exists() {
        panic!("Base model path in config doesn't exist");
    }
    let modelPath = modelsPath.join(&model.name);
    if !modelPath.exists() {
        panic!("Model in config doesn't exist");
    }

    let bpFilePath = modelPath.join("BPCheck.xml");
    if !bpFilePath.exists() {
        panic!("BPCheck.xml doesn't exist in model");
    }
    let xml = fs::read_to_string(bpFilePath).unwrap();
    let diags: Diagnostics = from_str(&xml).unwrap();
    diags.Items.Diagnostic
}
