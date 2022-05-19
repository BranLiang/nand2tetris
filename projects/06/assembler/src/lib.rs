mod parser;

use std::env::Args;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file = File::open(
        Path::new(&config.filename)
    )?;
    let parser = parser::Parser::new(file);
    let mut output = OpenOptions::new().write(true).truncate(true).create(true).open(
        Path::new(&config.destination)
    )?;
    
    for instruction in parser {
        writeln!(output, "{:016b}", instruction)?;
    }
    println!("Done!");
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
                if value.ends_with(".asm") {
                    value
                } else {
                    format!("{}.asm", value).to_string()
                }
            },
            None => return Err("missing filename")
        };
        let destination = filename.replace(".asm", ".hack");

        Ok(Config { filename, destination })
    }
}
