use std::{env, process};
use assembler::Config;
use assembler::run;


fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
       eprintln!("Error parsing arguments: {}", err);
       process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Error parsing aasembly file: {}", e);
        process::exit(1);
    }
}
