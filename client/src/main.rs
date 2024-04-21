mod routing;
mod tun;
#[macro_use]
mod errors;

use std::path::Path;

use errors::*;
use serde_derive::{Deserialize, Serialize};

// TODO: Custom close-out error handling

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fconf: FileConfig = toml::from_str(&std::fs::read_to_string(get_config_path()?)?)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct FileConfig {
    #[serde(rename = "General")]
    pub general: FileConfigGeneral,
    #[serde(rename = "Server")]
    pub servers: Vec<FileConfigServer>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileConfigGeneral {
    pub version: usize,
    pub server: usize,
    pub interface: Option<String>,
    pub features: Vec<String>,
    pub features_danger: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileConfigServer {
    pub lowport: u16,
    pub highport: u16,
    // NOTE: proto should pull from a list (maybe even a hashmap) of middlewares
    pub proto: String,
    pub ip: String,
}

fn get_config_path() -> Result<String, Box<dyn std::error::Error>> {
    let shlex = shellexpand::tilde(if cfg!(windows) {
        "~/AppData/Roaming/ACC/acc.toml"
    } else {
        "~/.config/acc.toml"
    })
    .to_string();
    if let Ok(v) = std::env::var("ACC_CONFIG") {
        if Path::new(&v).exists() {
            Ok(v.to_string())
        } else {
            enocnf!()
        }
    } else if Path::new(&shlex).exists() {
        Ok(shlex)
    } else if cfg!(unix) && Path::new("/etc/acc.toml").exists() {
        Ok("/etc/acc.toml".to_string())
    } else if cfg!(windows) && Path::new("C:/Program Files/ACC/acc.toml").exists() {
        Ok("C:/Program Files/Common Files/acc.toml".to_string())
    } else if cfg!(debug_assertions) && Path::new("./sample-config.toml").exists() {
        Ok("./sample-config.toml".to_string())
    } else {
        enocnf!()
    }
}
