use std::fs;
use std::io;
use std::str::Chars;

enum Token {
    Keyword(String),
    Symbol(char),
    Identifier(String),
    Int(i16),
    String(String)
}

const KEYWORDS: [&'static str; 20] = [
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
    "this"
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

struct Tokenizer {
    tokens: Vec<Token>
}

impl Tokenizer {
    pub fn new(path: &str) -> Result<Self, io::Error> {
        let mut tokens = Vec::new();
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let line = if let Some((non_comment, _comment)) = line.split_once("//") {
                non_comment
            } else {
                line
            };
            let line = Line::new(line.trim());
            for token in line {
                tokens.push(token);
            }
        }
        Ok(Self { tokens })
    }
}

impl Iterator for Tokenizer {
    type Item=Token;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

struct Line<'a> {
    chars: Chars<'a>,
    current_slice: String,
    current_is_string: bool,
    current_symbol: Option<char>
}

impl<'a> Line<'a> {
    pub fn new(line: &'a str) -> Self {
        Self {
            chars: line.chars(),
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

impl<'a> Iterator for Line<'a> {
    type Item=Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(symbol) = self.current_symbol {
            self.reset_current();
            return Some(Token::Symbol(symbol));
        }
        match self.chars.next() {
            Some(' ') => {
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
                self.current_slice.push(ch);
                self.next()
            },
            None => {
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
}