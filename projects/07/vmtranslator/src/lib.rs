use std::env::Args;
use std::error::Error;
use crate::parser::Command;

mod parser;
mod platform;

trait Translate {
    fn translate(&self, command: &Command) -> Option<String>;
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub struct Config {
    pub filename: String,
    pub destination: String
}

impl Config {
    pub fn new(mut args: Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(value) => {
                if value.ends_with(".vm") {
                    value
                } else {
                    format!("{}.vm", value).to_string()
                }
            },
            None => return Err("missing filename")
        };
        let destination = filename.replace(".vm", ".asm");

        Ok(Config { filename, destination })
    }
}