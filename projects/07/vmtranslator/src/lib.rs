use std::env::Args;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::parser::Command;

mod parser;
mod platform;

trait Translate {
    fn translate(&mut self, command: &Command) -> Option<String>;
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file = File::open(&config.filename)?;
    let parser = parser::Parser::new(file);
    let mut platform = platform::Hack::new(&config.filename);
    let mut output = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&config.destination)?;

    for command in parser {
        if let Some(assembly) = platform.translate(&command) {
            writeln!(output, "// {}", &command)?;
            write!(output, "{}", assembly)?;
        }
    }
    writeln!(output, "// Program end")?;
    write!(output, "{}", platform.end())?;
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