use std::{env, process};

use avrcomp::Config;

fn main() {
    let config = Config::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = avrcomp::run(config) {
        eprintln!("Application error: {e}");

        process::exit(1);
    }
}