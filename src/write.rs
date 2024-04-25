#![allow(non_snake_case)]

use std::{fs, path::Path};

use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::{config::Config, read::Diagnostic};
use xml::{reader::ParserConfig, writer::EmitterConfig};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct IgnoreDiagnostics {
    Name: String,
    Items: Items,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Items {
    Diagnostic: Vec<Diagnostic>,
}

pub fn write_diagnostics(
    data: &Vec<Diagnostic>,
    config: &Config,
    model: &String,
) -> Result<(), String> {
    let modelsPath = Path::new(&config.modelpath);
    if !modelsPath.exists() {
        return Err("Base model path in config doesn't exist".to_owned());
    }
    let modelPath = modelsPath.join(&model);
    if !modelPath.exists() {
        return Err("Model in config doesn't exist".to_owned());
    }

    let supp_file_path = modelPath
        .join(&model)
        .join("AxIgnoreDiagnosticList")
        .join(format!("{}_BPSuppressions.xml", &model));
    if !supp_file_path.exists() {
        return Err(format!(
            "Suppressions file doesn't exist in model: {}",
            supp_file_path.to_str().unwrap_or("N/A")
        ));
    }
    let xml = match fs::read_to_string(&supp_file_path) {
        Ok(xml) => xml,
        Err(_) => {
            return Err(format!(
                "Could not read {}",
                &supp_file_path.to_str().unwrap_or("N/A")
            ))
        }
    };
    let mut suppressions: IgnoreDiagnostics = match from_str(&xml) {
        Ok(xml) => xml,
        Err(_) => return Err("Could not parse suppression xml file".to_owned()),
    };

    for item in data {
        if item.Justification == "" {
            continue;
        }
        if let Some(supp) = suppressions
            .Items
            .Diagnostic
            .iter_mut()
            .find(|d| d.Path == item.Path && d.Moniker == item.Moniker)
        {
            supp.Justification = item.Justification.clone();
            // println!("Mod: {0:?}", supp);
        } else {
            let mut supp = item.clone();
            supp.Justification = item.Justification.clone();
            // println!("New: {0:?}", &supp);
            suppressions.Items.Diagnostic.push(supp);
        }
    }

    let xml = match to_string(&suppressions) {
        Ok(xml) => xml,
        Err(_) => {
            return Err("Could not serialize suppressions".to_owned());
        }
    };
    let xml = match format_xml(xml.as_bytes()) {
        Ok(xml) => xml,
        Err(_) => {
            return Err("Could not format suppressions".to_owned());
        }
    };
    match fs::write(&supp_file_path, xml) {
        Ok(_) => Ok(()),
        Err(_) => Err("Could not write suppressions file".to_owned()),
    }
}

fn format_xml(src: &[u8]) -> Result<String, xml::reader::Error> {
    let mut dest = Vec::new();
    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(false)
        .create_reader(src);
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .normalize_empty_elements(false)
        .autopad_comments(false)
        .create_writer(&mut dest);
    for event in reader {
        if let Some(event) = event?.as_writer_event() {
            writer.write(event).unwrap();
        }
    }
    Ok(String::from_utf8(dest).unwrap())
}
