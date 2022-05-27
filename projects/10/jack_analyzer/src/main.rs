use std::process;
use std::env;
use jack_analyzer::run;
use jack_analyzer::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error parsing configs: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Runtime error: {}", e);
        process::exit(1);
    }
}
