use std::path;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::BTreeMap;
use std::env;

use toml;

use failure::Error;
use failure::ResultExt;

#[derive(Debug, Deserialize)]
pub struct Server {
    #[serde(skip)]
    pub name: Option<String>,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub player_limit: i32,
    pub servers: BTreeMap<String, Server>,
}

const CONFIG_TOML: &str = r#"
host = "0.0.0.0:30001"
player_limit = -1

[servers]
  [servers.lobby]
    address = "localhost:30000"
"#;

pub fn init() {
    let current_path = env::current_dir().unwrap();
    let current_path = current_path.as_path();

    if !&current_path.join("config.toml").exists() {
        create_file(&current_path.join("config.toml"), CONFIG_TOML).expect("Could not create config.toml");
    }
}

pub fn read() -> Result<Config, Error> {
    let current_path = env::current_dir().unwrap();
    let current_path = current_path.as_path();
    let config_path = current_path.join("config.toml");

    if config_path.exists() {
        let mut contents = String::new();

        File::open(config_path)
            .context("Couldn't find config.toml file")?
            .read_to_string(&mut contents)
            .context("Unable to read config.toml file")?;

        match toml::from_str(&contents) {
            Ok(mut config) => {
                hydrate_config(&mut config);

                return Ok(config);
            }
            Err(e) => {
                return Err(format_err!("{}", e));
            }
        }

    } else {
        return Err(format_err!("Could not read config.toml"));
    }
}

fn hydrate_config(config: &mut Config) {
    for (server_name, server) in config.servers.iter_mut() {
        server.name = Some(server_name.to_string());
    }
}

fn create_file(path: &path::Path, content: &str) -> Result<(), Error> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;

    file.write_all(content.as_bytes()).unwrap();

    Ok(())
}