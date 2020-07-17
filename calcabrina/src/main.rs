use std::env;
use std::process;

use ff4::Rom;

use calcabrina::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while parsing arguments: {}", err);
        process::exit(1);
    });

    let rom = Rom::new(&config.filename).unwrap_or_else(|err| {
        println!("Error loading ROM file: {}", err);
        process::exit(1);
    });

    println!("ROM title: {}", rom.title());
}
