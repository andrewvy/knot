extern crate clap;
extern crate knot;
extern crate env_logger;

#[macro_use]
extern crate log;

use std::env;
use std::process::exit;

use clap::App;
use knot::{config, proxy};
use env_logger::Env;

const LOGGER_ENV_VAR_NAME: &'static str = "KNOT_LOGGER_LEVEL";

fn main() {
    match env::var(LOGGER_ENV_VAR_NAME) {
        Ok(_) => {},
        Err(_) => {
            env::set_var(LOGGER_ENV_VAR_NAME, "info");
        },
    }

    let env = Env::new().filter(LOGGER_ENV_VAR_NAME);

    env_logger::init_from_env(env);

    let app = App::new("knot")
        .version("0.1")
        .author("Andrew V. <andrew@andrewvy.com>");

    let _matches = app.get_matches();

    config::init();

    match config::read() {
        Ok(config) => {
            match config.validate() {
                Ok(_) => proxy::start(&config),
                Err(error) => {
                    error!("{}", error);
                    exit(1);
                }
            }
        },
        Err(error) => {
            error!("Could not load config.toml: {}", error);
            exit(1);
        }
    }
}
