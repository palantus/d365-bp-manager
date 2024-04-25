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
            supp_file_path.to_str().unwrap()
        ));
    }
    let xml = fs::read_to_string(&supp_file_path).unwrap();
    let mut suppressions: IgnoreDiagnostics = from_str(&xml).unwrap();

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

    match to_string(&suppressions) {
        Ok(xml) => {
            let xml = format_xml(xml.as_bytes()).unwrap();
            fs::write(&supp_file_path, xml).unwrap();
        }
        Err(err) => {
            dbg!(err);
        }
    }
    // println!("{0:?}", diags.Items);

    Ok(())
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
