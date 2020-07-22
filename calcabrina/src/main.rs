use std::env;
use std::process;

use calcabrina::config;

fn main() {
    let config = config::Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while parsing arguments: {}", err);
        process::exit(1);
    });

    calcabrina::run(config).unwrap_or_else(|err| {
        println!("Error: {}", err);
        process::exit(1);
    })
}
