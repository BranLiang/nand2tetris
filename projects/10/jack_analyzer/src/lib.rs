use std::env::Args;
use std::error::Error;
use std::fs::{OpenOptions, File, self};
use std::path::Path;

mod tokenizer;
mod parser;
mod utils;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.source {
        Source::File(filename) => {
            let mut output = output_file(&filename.replace(".jack", ".xml"));
            write_xml(&filename, &mut output)?;
        },
        Source::Directory(directory) => {
            let path = fs::read_dir(directory)?;
            for entry in path {
                let path = entry?.path();
                if path.extension().unwrap() == "jack" {
                    let output_filename = format!("{}", path.as_os_str().to_str().unwrap()).replace(".jack", ".xml");
                    let mut output = output_file(&output_filename);
                    write_xml(path.as_os_str().to_str().unwrap(), &mut output)?;
                }
            }
        }
    }
    Ok(())
}

fn write_xml(filename: &str, output: &mut File) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    parser::XML::compile(file, output)
}

fn output_file(path: &str) -> File {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path).unwrap()
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