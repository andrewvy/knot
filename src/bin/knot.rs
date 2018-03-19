extern crate clap;
extern crate knot;

use clap::App;
use knot::{config, proxy};

fn main() {
    let app = App::new("knot")
        .version("0.1")
        .author("Andrew V. <andrew@andrewvy.com>");

    let _matches = app.get_matches();

    config::init();

    match config::read() {
        Ok(config) => {
            proxy::start(&config);
        },
        Err(error) => {
            println!("Could not load config.toml: {}", error);
        }
    }
}
