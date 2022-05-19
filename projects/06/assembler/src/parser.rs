use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Lines;
use std::io::prelude::*;

pub enum Instruction {
    A(String),
    L(String),
    C { dest: Option<String>, comp: String, jump: Option<String> }
}

impl Instruction {
    pub fn to_decimal(&self, dictionary: &HashMap<String, i16>) -> Option<i16> {
        match &self {
            &Instruction::A(symbol) => {
                if let Ok(address) = symbol.parse::<i16>() {
                    Some(address)
                } else {
                    let address = dictionary.get(symbol).unwrap();
                    Some(*address)
                }
            },
            &Instruction::L(_symbol) => {
                None
            },
            &Instruction::C { dest, comp, jump } => {
                let opcode_b: i16 = 0b111 << 13;
                let comp_b: i16 = match comp.as_str() {
                    "0" => 0b0101010,
                    "1" => 0b0111111,
                    "-1" => 0b0111010,
                    "D" => 0b0001100,
                    "A" => 0b0110000,
                    "M" => 0b1110000,
                    "!D" => 0b0001101,
                    "!A" => 0b0110001,
                    "!M" => 0b1110001,
                    "-D" => 0b0001111,
                    "-A" => 0b0110011,
                    "-M" => 0b1110011,
                    "D+1" | "1+D" => 0b0011111,
                    "A+1" | "1+A" => 0b0110111,
                    "M+1" | "1+M" => 0b1110111,
                    "D-1" => 0b0001110,
                    "A-1" => 0b0110010,
                    "M-1" => 0b1110010,
                    "D+A" | "A+D" => 0b0000010,
                    "D+M" | "M+D" => 0b1000010,
                    "D-A" => 0b0010011,
                    "D-M" => 0b1010011,
                    "A-D" => 0b0000111,
                    "M-D" => 0b1000111,
                    "D&A" | "A&D" => 0b0000000,
                    "D&M" | "M&D" => 0b1000000,
                    "D|A" | "A|D" => 0b0010101,
                    "D|M" | "M|D" => 0b1010101,
                    _ => panic!("Invalid comp: {}", comp)
                } << 6;
                let dest_b: i16 = if let Some(v) = dest {
                    match v.as_ref() {
                        "M" => 0b001,
                        "D" => 0b010,
                        "DM" | "MD" => 0b011,
                        "A" => 0b100,
                        "AM" | "MA" => 0b101,
                        "AD" | "DA" => 0b110,
                        "ADM" | "AMD" | "DAM" | "DMA" | "MAD" | "MDA" => 0b111,
                        _ => panic!("Invalid dest: {}", v)
                    }
                } else {
                    0b000
                } << 3;
                let jump_b: i16 = if let Some(v) = jump {
                    match v.as_ref() {
                        "JGT" => 0b001,
                        "JEQ" => 0b010,
                        "JGE" => 0b011,
                        "JLT" => 0b100,
                        "JNE" => 0b101,
                        "JLE" => 0b110,
                        "JMP" => 0b111,
                        _ => panic!("Invalid jump")
                    }
                } else {
                    0b000
                };
                let binary = opcode_b | comp_b | dest_b | jump_b;
                Some(binary)
            }
        }
    }
}

pub struct Parser<'a> {
    lines: Lines<BufReader<&'a File>>
}

impl<'a> Parser<'a> {
    pub fn new(file: &'a File) -> Self {
        let lines = BufReader::new(file).lines();
        Parser { lines }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?.unwrap();
        line_to_instruction(&line).or_else(|| self.next())
    }
}

fn line_to_instruction(line: &str) -> Option<Instruction> {
    let line = if let Some((line_without_comment, _comment)) = line.split_once("//") {
        line_without_comment
    } else {
        line
    };
    let line = line.trim();
    if line.starts_with("//") || line.is_empty() {
        return None;
    }
    // Instruction A
    if line.starts_with('@') {
        let symbol = line.strip_prefix('@').unwrap();
        return Some(Instruction::A(symbol.to_string()));
    }
    // Instruction L
    if line.starts_with('(') && line.ends_with(')') {
        let symbol = line
            .strip_prefix('(').unwrap()
            .strip_suffix(')').unwrap();
        return Some(Instruction::L(symbol.to_string()));
    }
    // Instruction C
    match line.split_once('=') {
        Some((dest, other)) => {
            match other.split_once(';') {
                Some((comp, jump)) => {
                    return Some(Instruction::C {
                        dest: Some(dest.to_string()),
                        comp: comp.to_string(),
                        jump: Some(jump.to_string())
                    });
                },
                None => {
                    return Some(Instruction::C {
                        dest: Some(dest.to_string()),
                        comp: other.to_string(),
                        jump: None
                    });
                }
            }
        },
        None => {
            match line.split_once(';') {
                Some((comp, jump)) => {
                    return Some(Instruction::C {
                        dest: None,
                        comp: comp.to_string(),
                        jump: Some(jump.to_string())
                    });
                },
                None => {
                    return Some(Instruction::C {
                        dest: None,
                        comp: line.to_string(),
                        jump: None
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempfile;
    use std::io::SeekFrom;

    fn fixture(content: &str) -> File {
        let mut file = tempfile().unwrap();
        for line in content.lines() {
            writeln!(file, "{}", line).unwrap();
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        file
    }

    #[test]
    fn instruction_a_to_binary() {
        let dictionary = HashMap::new();

        let a1 = Instruction::A("17".to_string());
        assert_eq!("0000000000010001", format!("{:016b}", a1.to_decimal(&dictionary).unwrap()));

        let a2 = Instruction::A("1".to_string());
        assert_eq!("0000000000000001", format!("{:016b}", a2.to_decimal(&dictionary).unwrap()))
    }

    #[test]
    fn instruction_c_to_binary() {
        let dictionary = HashMap::new();

        let c1 = Instruction::C { dest: None, comp: "0".to_string(), jump: None };
        assert_eq!("1110101010000000", format!("{:016b}", c1.to_decimal(&dictionary).unwrap()));

        let c2 = Instruction::C { dest: None, comp: "M".to_string(), jump: None };
        assert_eq!("1111110000000000", format!("{:016b}", c2.to_decimal(&dictionary).unwrap()));

        let c3 = Instruction::C { dest: Some("D".to_string()), comp: "D+M".to_string(), jump: None };
        assert_eq!("1111000010010000", format!("{:016b}", c3.to_decimal(&dictionary).unwrap()));

        let c4 = Instruction::C { dest: None, comp: "D".to_string(), jump: Some("JGE".to_string()) };
        assert_eq!("1110001100000011", format!("{:016b}", c4.to_decimal(&dictionary).unwrap()));

        let c5 = Instruction::C { dest: Some("D".to_string()), comp: "D+M".to_string(), jump: Some("JGT".to_string()) };
        assert_eq!("1111000010010001", format!("{:016b}", c5.to_decimal(&dictionary).unwrap()));
    }

    #[test]
    fn lines_ignored() {
        let comment = "// I am a comment";
        assert!(line_to_instruction(comment).is_none());

        let blank = "       ";
        assert!(line_to_instruction(blank).is_none());
    }

    #[test]
    fn lines_to_a_instruction() {
        let a1 = line_to_instruction("@n").unwrap();
        match a1 {
            Instruction::A(symbol) => assert_eq!(symbol, "n"),
            _ => panic!("instruction parsing error")
        }

        let a2 = line_to_instruction("@17").unwrap();
        match a2 {
            Instruction::A(symbol) => assert_eq!(symbol, "17"),
            _ => panic!("instruction parsing error")
        }
    }

    #[test]
    fn lines_to_l_instruction() {
        let l = line_to_instruction("(LOOP)").unwrap();
        match l {
            Instruction::L(symbol) => assert_eq!(symbol, "LOOP"),
            _ => panic!("instruction parsing error")
        }
    }

    #[test]
    fn lines_to_c_instruction() {
        let dictionary = HashMap::new();

        let c1 = line_to_instruction("0").unwrap();
        assert_eq!("1110101010000000", format!("{:016b}", c1.to_decimal(&dictionary).unwrap()));

        let c2 = line_to_instruction("M").unwrap();
        assert_eq!("1111110000000000", format!("{:016b}", c2.to_decimal(&dictionary).unwrap()));

        let c3 = line_to_instruction("D=D+M").unwrap();
        assert_eq!("1111000010010000", format!("{:016b}", c3.to_decimal(&dictionary).unwrap()));

        let c4 = line_to_instruction("D;JGE").unwrap();
        assert_eq!("1110001100000011", format!("{:016b}", c4.to_decimal(&dictionary).unwrap()));

        let c5 = line_to_instruction("D=D+M;JGT").unwrap();
        assert_eq!("1111000010010001", format!("{:016b}", c5.to_decimal(&dictionary).unwrap()));
    }

    #[test]
    fn test_basic_parser() {
        let dictionary = HashMap::new();
        let content = "\
// Computes R0 = 2 + 3  (R0 refers to RAM[0])

@2
D=A
@3
D=D+A
@0
M=D";
        let file = fixture(content);
        let mut parser = Parser::new(&file);
        let i1 = parser.next().unwrap();
        assert_eq!("0000000000000010", format!("{:016b}", i1.to_decimal(&dictionary).unwrap()));

        let i2 = parser.next().unwrap();
        assert_eq!("1110110000010000", format!("{:016b}", i2.to_decimal(&dictionary).unwrap()));

        let i3 = parser.next().unwrap();
        assert_eq!("0000000000000011", format!("{:016b}", i3.to_decimal(&dictionary).unwrap()));

        let i4 = parser.next().unwrap();
        assert_eq!("1110000010010000", format!("{:016b}", i4.to_decimal(&dictionary).unwrap()));

        let i5 = parser.next().unwrap();
        assert_eq!("0000000000000000", format!("{:016b}", i5.to_decimal(&dictionary).unwrap()));

        let i6 = parser.next().unwrap();
        assert_eq!("1110001100001000", format!("{:016b}", i6.to_decimal(&dictionary).unwrap()));

        assert!(parser.next().is_none());
    }
}