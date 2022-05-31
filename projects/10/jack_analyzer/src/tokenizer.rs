use std::io;
use std::io::BufRead;
use std::io::Lines;
use std::io::BufReader;
use std::fs::File;

#[derive(Debug, Clone)]
pub enum Token {
    Keyword(String),
    Symbol(char),
    Identifier(String),
    Int(i16),
    String(String)
}

const KEYWORDS: [&'static str; 21] = [
    "class",
    "method",
    "function",
    "constructor",
    "int",
    "boolean",
    "char",
    "void",
    "var",
    "static",
    "field",
    "let",
    "do",
    "if",
    "else",
    "return",
    "true",
    "false",
    "null",
    "this",
    "while"
];

const SYMBOLS: [char; 19] = [
    '{',
    '}',
    '(',
    ')',
    '[',
    ']',
    '.',
    ',',
    ';',
    '+',
    '-',
    '*',
    '/',
    '&',
    '|',
    '<',
    '>',
    '=',
    '~'
];

#[derive(Debug)]
pub struct Tokenizer {
    lines: Lines<BufReader<File>>,
    current_line: Line,
    is_comment: bool
}

impl Tokenizer {
    pub fn new(file: File) -> Result<Self, io::Error> {
        let lines = BufReader::new(file).lines();
        let current_line = Line::new("");
        Ok(Self { lines, current_line, is_comment: false })
    }
}

impl Iterator for Tokenizer {
    type Item=Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.current_line.next() {
            return Some(token);
        } else {
            let line = self.lines.next()?.unwrap();
            let line = line.trim();

            // handle /** comments */
            if line.starts_with("/** ") && line.ends_with(" */") {
                return self.next();
            } else if line.starts_with("/**") {
                self.is_comment = true;
                return self.next();
            } else if line.starts_with("*/") {
                self.is_comment = false;
                return self.next();
            } else if self.is_comment {
                return self.next();
            }

            let line = if let Some((non_comment, _comment)) = line.split_once("//") {
                non_comment
            } else {
                line
            };
            self.current_line = Line::new(line);
            self.next()
        }
    }
}

#[derive(Debug)]
struct Line {
    raw_line: String,
    index: usize,
    current_slice: String,
    current_is_string: bool,
    current_symbol: Option<char>
}

impl Line {
    pub fn new(line: &str) -> Self {
        Self {
            raw_line: line.to_string(),
            index: 0,
            current_slice: String::new(),
            current_is_string: false,
            current_symbol: None
        }
    }

    pub fn token(&self) -> Token {
        let slice = self.current_slice.clone();
        if self.current_is_string {
            return Token::String(slice);
        }
        if let Some(symbol) = self.current_symbol {
            return Token::Symbol(symbol);
        }
        if KEYWORDS.contains(&&slice[..]) {
            return Token::Keyword(slice);
        }
        if slice.chars().all(|ch| ch.is_numeric()) {
            let num: i16 = slice.parse().unwrap();
            return Token::Int(num);
        }
        Token::Identifier(slice)
    }

    fn reset_current(&mut self) {
        self.current_slice = "".to_string();
        self.current_is_string = false;
        self.current_symbol = None;
    }
}

impl Iterator for Line {
    type Item=Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(symbol) = self.current_symbol {
            self.reset_current();
            return Some(Token::Symbol(symbol));
        }
        let n = self.index;
        let char = self.raw_line.chars().nth(n);
        match char {
            Some(' ') => {
                self.index += 1;
                if self.current_is_string {
                    self.current_slice.push(' ');
                    self.next()
                } else if self.current_slice.len() > 0 {
                    let token = self.token();
                    self.reset_current();
                    Some(token)
                } else {
                    self.next()
                }
            },
            Some('"') => {
                self.index += 1;
                if self.current_slice.is_empty() {
                    self.current_is_string = true;
                    self.next()
                } else {
                    let token = self.token();
                    self.reset_current();
                    Some(token)
                }
            },
            Some(ch) if SYMBOLS.contains(&ch) => {
                self.index += 1;
                if self.current_is_string {
                    self.current_slice.push(ch);
                    self.next()
                } else if self.current_slice.len() > 0 {
                    let token = self.token();
                    self.reset_current();
                    self.current_symbol = Some(ch);
                    Some(token)
                } else {
                    self.current_symbol = Some(ch);
                    self.next()
                }
            },
            Some(ch) => {
                self.index += 1;
                self.current_slice.push(ch);
                self.next()
            },
            None => {
                self.index += 1;
                if self.current_slice.is_empty() {
                    None
                } else {
                    let token = self.token();
                    self.reset_current();
                    Some(token)
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
    use std::io::prelude::*;

    fn fixture(content: &str) -> File {
        let mut file = tempfile().unwrap();
        for line in content.lines() {
            writeln!(file, "{}", line).unwrap();
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        file
    }

    #[test]
    fn test_line() {
        let line = "do Output.printString(\"The average is \");  let i = 1;";
        let mut line = Line::new(line);

        match line.next().unwrap() {
            Token::Keyword(k) if k == "do".to_string() => {},
            _ => panic!("failed to parse keyword `do`")
        }

        match line.next().unwrap() {
            Token::Identifier(v) if v == "Output".to_string() => {},
            _ => panic!("failed to parse identifier `Output`")
        }

        match line.next().unwrap() {
            Token::Symbol('.') => {},
            _ => panic!("failed to parse the symbol `.`")
        }

        match line.next().unwrap() {
            Token::Identifier(v) if v == "printString".to_string() => {},
            _ => panic!("failed to parse identifier `printString`")
        }

        match line.next().unwrap() {
            Token::Symbol('(') => {},
            _ => panic!("failed to parse the symbol `(`")
        }

        match line.next().unwrap() {
            Token::String(v) if v == "The average is ".to_string() => {},
            Token::String(v) => panic!("failed to parse the string content: {}", v),
            _ => panic!("Unknown string parsing error")
        }

        match line.next().unwrap() {
            Token::Symbol(')') => {},
            _ => panic!("failed to parse the symbol `)`")
        }

        match line.next().unwrap() {
            Token::Symbol(';') => {},
            _ => panic!("failed to parse the symbol `;`")
        }

        match line.next().unwrap() {
            Token::Keyword(k) if k == "let".to_string() => {},
            _ => panic!("failed to parse keyword `let`")
        }

        match line.next().unwrap() {
            Token::Identifier(v) if v == "i".to_string() => {},
            _ => panic!("failed to parse identifier `i`")
        }

        match line.next().unwrap() {
            Token::Symbol('=') => {},
            _ => panic!("failed to parse the symbol `=`")
        }

        match line.next().unwrap() {
            Token::Int(1) => {},
            _ => panic!("failed to parse the int `1`")
        }

        match line.next().unwrap() {
            Token::Symbol(';') => {},
            _ => panic!("failed to parse the symbol `;`")
        }

        assert!(line.next().is_none());
    }

    #[test]
    fn test_tokenizer() {
        let content = "\
            if (x < 0) {
                // print the slogan
                /** I am a comment */
                /**
                 * Test
                 */
                do Output.printString(\"hello world :)\");
            }
        ";
        let file = fixture(content);
        let mut tokenizer = Tokenizer::new(file).unwrap();

        match tokenizer.next() {
            Some(Token::Keyword(v)) if v == "if".to_string() => {},
            _ => panic!("error parsing keyword `if`")
        }

        match tokenizer.next() {
            Some(Token::Symbol('(')) => {},
            _ => panic!("error parsing symbol `(`")
        }

        match tokenizer.next() {
            Some(Token::Identifier(v)) if v == "x".to_string() => {},
            _ => panic!("error parsing identifier `x`")
        }

        match tokenizer.next() {
            Some(Token::Symbol('<')) => {},
            _ => panic!("error parsing symbol `<`")
        }

        match tokenizer.next() {
            Some(Token::Int(0)) => {},
            _ => panic!("error parsing integer `0`")
        }

        match tokenizer.next() {
            Some(Token::Symbol(')')) => {},
            _ => panic!("error parsing symbol `)`")
        }

        match tokenizer.next() {
            Some(Token::Symbol('{')) => {},
            _ => panic!("error parsing symbol `{{`")
        }

        match tokenizer.next() {
            Some(Token::Keyword(v)) if v == "do".to_string() => {},
            Some(token) => panic!("error parsing: {:?}", token),
            _ => panic!("error parsing keyword `do`")
        }

        match tokenizer.next() {
            Some(Token::Identifier(v)) if v == "Output".to_string() => {},
            _ => panic!("error parsing identifier `Output`")
        }

        match tokenizer.next() {
            Some(Token::Symbol('.')) => {},
            _ => panic!("error parsing symbol `.`")
        }

        match tokenizer.next() {
            Some(Token::Identifier(v)) if v == "printString".to_string() => {},
            _ => panic!("error parsing identifier `printString`")
        }

        match tokenizer.next() {
            Some(Token::Symbol('(')) => {},
            _ => panic!("error parsing symbol `(`")
        }

        match tokenizer.next() {
            Some(Token::String(v)) if v == "hello world :)".to_string() => {},
            _ => panic!("error parsing string")
        }

        match tokenizer.next() {
            Some(Token::Symbol(')')) => {},
            _ => panic!("error parsing symbol `)`")
        }

        match tokenizer.next() {
            Some(Token::Symbol(';')) => {},
            _ => panic!("error parsing symbol `;`")
        }

        match tokenizer.next() {
            Some(Token::Symbol('}')) => {},
            _ => panic!("error parsing symbol `}}`")
        }

        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn test() {
        assert!(" */\n".trim().starts_with("*/"));
    }
}