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
            match config.target {
                Target::XML => {
                    let mut output = output_file(&filename.replace(".jack", ".xml"));
                    write_xml(&filename, &mut output)?;
                },
                Target::VM => {
                    let mut output = output_file(&filename.replace(".jack", ".vm"));
                    write_vm(&filename, &mut output)?;
                }
            }
        },
        Source::Directory(directory) => {
            let path = fs::read_dir(directory)?;
            for entry in path {
                let path = entry?.path();
                if path.extension().unwrap() == "jack" {
                    match config.target {
                        Target::XML => {
                            let output_filename = format!("{}", path.as_os_str().to_str().unwrap()).replace(".jack", ".xml");
                            let mut output = output_file(&output_filename);
                            write_xml(path.as_os_str().to_str().unwrap(), &mut output)?;
                        },
                        Target::VM => {
                            let output_filename = format!("{}", path.as_os_str().to_str().unwrap()).replace(".jack", ".vm");
                            let mut output = output_file(&output_filename);
                            write_vm(path.as_os_str().to_str().unwrap(), &mut output)?;
                        }
                    }
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

fn write_vm(filename: &str, output: &mut File) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    parser::VM::compile(file, output)
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

enum Target {
    XML,
    VM
}

pub struct Config {
    source: Source,
    target: Target
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

        let target = match args.next() {
            Some(v) => {
                if v == "xml".to_string() {
                    Target::XML
                } else {
                    Target::VM
                }
            },
            None => Target::VM
        };

        Ok(Config { source, target })
    }
}