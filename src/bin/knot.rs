extern crate clap;
extern crate knot;

use clap::{App, Arg};
use knot::proxy;

fn main() {
    let app = App::new("knot")
        .version("0.1")
        .author("Andrew V. <andrew@andrewvy.com>")
        .arg(
            Arg::with_name("addr")
            .short("b")
            .takes_value(true)
            .required(true)
        )
        .arg(
            Arg::with_name("remote-host")
            .short("h")
            .takes_value(true)
            .required(true)
        );

    let matches = app.get_matches();
    let addr = matches.value_of("addr").expect("No addr specified");
    let remote_host = matches.value_of("remote-host").expect("No remote host specified");

    proxy::start(addr, remote_host);
}
