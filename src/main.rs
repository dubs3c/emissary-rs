use clap::Parser;
use ini::Ini;
use reqwest::blocking::Client;
use reqwest::Error as reqwestError;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::path::PathBuf;

mod tui;

#[derive(Serialize)]
#[serde(untagged)]
enum MyType {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

impl MyType {
    fn to_string(&self) -> String {
        match self {
            MyType::Int(i) => i.to_string(),
            MyType::Float(f) => f.to_string(),
            MyType::Str(s) => s.clone(),
            MyType::Bool(b) => b.to_string(),
        }
    }
}

fn send(webhook: &str, json: HashMap<String, MyType>) -> Result<(), reqwestError> {
    Client::new().post(webhook).json(&json).send()?;
    Ok(())
}

fn prepare(message: &str, config: PathBuf) -> Result<HashMap<String, MyType>, Box<dyn Error>> {
    let conf = Ini::load_from_file(config)?;

    // TODO: Perform better error handling, maybe create a new error type?
    let default_section = conf.section(Some("Default")).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Default section not found")
    })?;
    let default_channel = default_section.get("channel").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Channel key not found in default section",
        )
    })?;

    let channel = conf.section(Some(default_channel)).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "channel section not found")
    })?;
    let channel_webhook = channel
        .get("webhook")
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "channel section not found")
        })?
        .to_string();
    let text_field = channel
        .get("textField")
        .map_or("message".to_owned(), |s| s.to_owned());
    let data = channel.get("data");

    let mut json = HashMap::new();

    json.insert(text_field, MyType::Str(message.to_string()));
    json.insert(String::from("webhook"), MyType::Str(channel_webhook));

    // Dynamically parse custom json structure
    if let Some(parsed_data) = data {
        let key_value = serde_json::from_str::<Value>(parsed_data).expect("could not parse data");
        if let Some(obj) = key_value.as_object() {
            for (key, val) in obj {
                match val {
                    Value::String(s) => {
                        json.insert(key.clone(), MyType::Str(s.to_string()));
                    }
                    Value::Number(num) => {
                        // Convert numbers to string
                        if let Some(n) = num.as_i64() {
                            json.insert(key.clone(), MyType::Int(n));
                        } else if let Some(f) = num.as_f64() {
                            json.insert(key.clone(), MyType::Float(f));
                        }
                    }
                    Value::Bool(b) => {
                        json.insert(key.clone(), MyType::Bool(b.clone()));
                    }
                    // Handle other types as needed, or ignore them
                    _ => (), // TODO: Perform better error handling
                }
            }
        }
    }

    Ok(json)
}

fn main() {
    let mut config_location = PathBuf::new();
    if let Some(home) = dirs::home_dir() {
        let mut config_path = home;
        config_path.push(".config/emissary.ini");
        config_location = config_path;
    } else {
        eprintln!("[-] Failed getting user home folder")
    }

    let args = tui::Args::parse();
    let msg = if let Some(value) = args.msg {
        value // If args.msg has a value, use it
    } else {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error taking input from stdin"); // TODO: Perform better error handling
        input.trim().to_string()
    };

    match prepare(&msg, config_location) {
        Ok(jay) => {
            if let Some(w) = jay.get("webhook") {
                match send(w.to_string().as_ref(), jay) {
                    Ok(..) => println!("[+] Message Sent!"),
                    Err(e) => println!("[-] Error sending message: {}", e),
                }
            } else {
                println!("[-] Webhook was not set...")
            }
        }
        Err(e) => println!("[-] Error sending message: {}", e),
    }
}
