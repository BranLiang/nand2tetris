use std::error::Error;
use std::fs::{File, OpenOptions, self};
use std::io::Write;
use std::path::Path;
use crate::parser::Command;

mod parser;
mod platform;

trait Translate {
    fn translate(&mut self, command: &Command) -> Option<String>;
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut output = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&config.destination)?;
    match config.source {
        Source::File(filename) => {
            handle_file(&filename, &mut output)?;
        },
        Source::Directory(directory) => {
            let path = fs::read_dir(directory)?;
            for entry in path {
                let path = entry?.path();
                if path.ends_with(".vm") {
                    handle_file(path.file_name().unwrap().to_str().unwrap(), &mut output)?;
                }
            }
        }
    }
    writeln!(output, "// Program end")?;
    write!(output, "{}", platform::Hack::end())?;
    Ok(())
}

fn handle_file(filename: &str, output: &mut File) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    let parser = parser::Parser::new(file);
    let mut platform = platform::Hack::new(filename);
    for command in parser {
        if let Some(assembly) = platform.translate(&command) {
            writeln!(output, "// {}", &command)?;
            write!(output, "{}", assembly)?;
        }
    }
    Ok(())
}

pub enum Source {
    File(String),
    Directory(String)
}

pub struct Config {
    pub source: Source,
    pub destination: String
}

impl Config {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let source = match args.next() {
            Some(value) if value.ends_with(".vm") => {
                Source::File(value)
            },
            Some(value) if value.ends_with('/') => {
                Source::Directory(value)
            },
            Some(_value) => {
                return Err("Invalid source")
            },
            None => return Err("missing filename")
        };
        
        let destination = match &source {
            Source::File(filename) => {
                filename.replace(".vm", ".asm")
            },
            Source::Directory(path) => {
                let mut path = path.clone();
                let mut directory = String::new();
                for component in Path::new(&path).iter() {
                    directory = component.to_str().unwrap().to_string()
                }
                let filename = format!("{}.asm", directory);
                path.push_str(&filename);
                path
            }
        };

        Ok(Config { source, destination })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_source() {
        let args = vec!["app".to_string(), "../myfolder/test.vm".to_string()];
        let config = Config::new(args.into_iter()).unwrap();
        match config.source {
            Source::File(filename) if filename == "../myfolder/test.vm".to_string() => {},
            _ => panic!("Fail to parse the file input source!")
        }
        match config.destination {
            value if value == "../myfolder/test.asm".to_string() => {},
            _ => panic!("Fail to parse the file destination source!")
        }
    }

    #[test]
    fn directory_source() {
        let args = vec!["app".to_string(), "../myfolder/".to_string()];
        let config = Config::new(args.into_iter()).unwrap();
        match config.source {
            Source::Directory(path) if path == "../myfolder/".to_string() => {},
            _ => panic!("Fail to parse the directory input source!")
        }
        match config.destination {
            value if value == "../myfolder/myfolder.asm".to_string() => {},
            _ => panic!("Fail to parse the directory destination source!")
        }
    }
}