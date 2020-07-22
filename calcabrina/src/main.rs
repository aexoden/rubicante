use std::env;
use std::process;

use log::error;

use calcabrina::config;
use calcabrina::util;

fn main() {
    util::setup_logger().unwrap_or_else(|err| {
        println!("FATAL: Could not initialize logger: {}", err);
    });

    let config = config::Config::new(env::args()).unwrap_or_else(|err| {
        error!("Error while parsing arguments: {}", err);
        process::exit(1);
    });

    calcabrina::run(config).unwrap_or_else(|err| {
        error!("Error running application: {}", err);
        process::exit(1);
    })
}
