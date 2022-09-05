mod mintd;
mod common;
mod repository;
mod queue;

use clap::{App, Arg};

fn main() {
    let matches = App::new("mint")
        .version("0.0.1")
        .author("roxy")
        .about("message queue in rust")
        .arg(
            Arg::with_name("serve")
                .help("start serve")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("port")
                .long("--port")
                .required(false)
                .default_value(":8090")
        )
        .get_matches();

    if matches.is_present("serve") {
        println!("hello world")
    }

    if let Some(port) = matches.value_of("port") {
        println!("get port {}", port)
    }
}