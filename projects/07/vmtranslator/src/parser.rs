use std::collections::HashMap;
use std::io::Lines;
use std::io::BufReader;
use std::fs::File;

pub enum Segment {
    Argument,
    Local,
    Static,
    This,
    That,
    Constant,
    Pointer,
    Temp,
}

pub enum Operator {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

pub enum Command {
    Arithmetic(Operator),
    Push(Segment, i16),
    Pop(Segment, i16),
}

pub struct Parser {
    lines: Lines<BufReader<File>>
}

impl Iterator for Parser {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?.unwrap();
        line_to_command(&line).or_else(|| self.next())
    }
}

fn line_to_command(line: &str) -> Option<Command> {
    // Remove comments
    let line = if let Some((non_comment, _comment)) = line.split_once("//") {
        non_comment
    } else {
        line
    };

    let mut line = line.trim().split_whitespace();
    match line.next() {
        Some("add") => Some(Command::Arithmetic(Operator::Add)),
        Some("sub") => Some(Command::Arithmetic(Operator::Sub)),
        Some("neg") => Some(Command::Arithmetic(Operator::Neg)),
        Some("eq") => Some(Command::Arithmetic(Operator::Eq)),
        Some("gt") => Some(Command::Arithmetic(Operator::Gt)),
        Some("lt") => Some(Command::Arithmetic(Operator::Lt)),
        Some("and") => Some(Command::Arithmetic(Operator::And)),
        Some("or") => Some(Command::Arithmetic(Operator::Or)),
        Some("not") => Some(Command::Arithmetic(Operator::Not)),
        Some("push") => {
            let subcommand = line.next()?;
            let segment = subcommand_to_segment(subcommand)?;
            let index = line.next()?;
            if let Ok(index) = index.parse::<i16>() {
                Some(Command::Push(segment, index))
            } else {
                None
            }
        },
        Some("pop") => {
            let subcommand = line.next()?;
            let segment = subcommand_to_segment(subcommand)?;
            let index = line.next()?;
            if let Ok(index) = index.parse::<i16>() {
                Some(Command::Pop(segment, index))
            } else {
                None
            }
        },
        _ => None
    }
    
}

fn subcommand_to_segment(subcommand: &str) -> Option<Segment> {
    match subcommand {
        "argument" => Some(Segment::Argument),
        "local" => Some(Segment::Local),
        "static" => Some(Segment::Static),
        "this" => Some(Segment::This),
        "that" => Some(Segment::That),
        "constant" => Some(Segment::Constant),
        "pointer" => Some(Segment::Pointer),
        "temp" => Some(Segment::Temp),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_line_to_command() {
        let command = line_to_command("add").unwrap();
        match command {
            Command::Arithmetic(Operator::Add) => {},
            _ => panic!("error parsing `add`!")
        }

        let command = line_to_command("or").unwrap();
        match command {
            Command::Arithmetic(Operator::Or) => {},
            _ => panic!("error parsing `or`!")
        }
    }

    #[test]
    fn push_line_to_command() {
        let line = "push constant 1";
        let command = line_to_command(line).unwrap();
        match command {
            Command::Push(Segment::Constant, 1) => {},
            _ => panic!("error parsing `{}`", line)
        }
    }

    #[test]
    fn pop_line_to_command() {
        let line = "pop local 2";
        let command = line_to_command(line).unwrap();
        match command {
            Command::Pop(Segment::Local, 2) => {},
            _ => panic!("error parsing `{}`", line)
        }
    }
}