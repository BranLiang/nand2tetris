mod parser;

use std::collections::HashMap;
use std::env::Args;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;

use crate::parser::Instruction;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(
        Path::new(&config.filename)
    )?;
    let parser = parser::Parser::new(&file);

    // Line counter
    let mut counter = 0i16;
    // Memory counter
    let mut m_address = 15i16;

    // Dictionary
    // Predefined symbols
    let mut dictionary = HashMap::new();
    for n in 0..16 {
        let key = format!("R{}", n);
        dictionary.insert(key, n);
    }
    dictionary.insert("SCREEN".to_string(), 16384i16);
    dictionary.insert("KBD".to_string(), 24576);
    dictionary.insert("SP".to_string(), 0);
    dictionary.insert("LCL".to_string(), 1);
    dictionary.insert("ARG".to_string(), 2);
    dictionary.insert("THIS".to_string(), 3);
    dictionary.insert("THAT".to_string(), 4);
    // Label symbols
    for instruction in parser {
        match instruction {
            Instruction::L(symbol) => {
                dictionary.entry(symbol).or_insert(counter);
            },
            _ => counter += 1
        }
    }
    // Variable symbols
    file.seek(SeekFrom::Start(0)).unwrap();
    let parser = parser::Parser::new(&file);
    for instruction in parser {
        match instruction {
            Instruction::A(symbol) => {
                if symbol.parse::<i16>().is_err() {
                    dictionary.entry(symbol).or_insert_with(|| {
                        m_address += 1;
                        m_address
                    });
                }
            },
            _ => {}
        }
    }

    let mut output = OpenOptions::new().write(true).truncate(true).create(true).open(
        Path::new(&config.destination)
    )?;
    
    file.seek(SeekFrom::Start(0)).unwrap();
    let parser = parser::Parser::new(&file);
    for instruction in parser {
        if let Some(address) = instruction.to_decimal(&dictionary) {
            writeln!(output, "{:016b}", address)?;
        }
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
