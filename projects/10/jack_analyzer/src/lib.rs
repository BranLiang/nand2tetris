use std::env::Args;
use std::error::Error;
use std::path::Path;

mod tokenizer;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}

enum Source {
    File(String),
    Directory(String)
}

pub struct Config {
    source: Source
}

impl Config {
    pub fn new(mut args: Args) -> Result<Self, &'static str> {
        args.next();

        let source = match args.next() {
            Some(file) if file.ends_with(".jack") && Path::new(&file).exists() => {
                Source::File(file)
            },
            Some(directory) if Path::new(&directory).is_dir() => {
                Source::Directory(directory)
            },
            None => return Err("Missing filename or directory."),
            _ => return Err("Invalid filename or directory.")
        };

        Ok(Config { source })
    }
}