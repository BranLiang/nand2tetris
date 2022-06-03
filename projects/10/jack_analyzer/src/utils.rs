use std::collections::HashMap;

use crate::parser::Type;

pub struct Padding(usize);

impl Padding {
    pub fn new() -> Self {
        Padding(0)
    }

    pub fn to_spaces(&self) -> String {
        vec![" "; self.0].concat()
    }

    pub fn increment(&mut self) -> &Self {
        self.0 += 2;
        self
    }

    pub fn decrement(&mut self) -> &Self {
        self.0 -= 2;
        self
    }
}

pub enum SymbolKind {
    Field,
    Static,
    Local,
    Argument
}

pub struct Symbol {
    var_name: String,
    var_type: Type,
    kind: SymbolKind,
    index: i16,
}

impl Symbol {
    pub fn vm_memory_segment(&self) -> String {
        match self.kind {
            SymbolKind::Field => "this".to_string(),
            SymbolKind::Argument => "argument".to_string(),
            SymbolKind::Local => "local".to_string(),
            SymbolKind::Static => "static".to_string()
        }
    }

    pub fn index(&self) -> i16 {
        self.index
    }

    pub fn class_name(&self) -> String {
        match &self.var_type {
            Type::ClassName(v) => v.to_string(),
            _ => panic!()
        }
    }
}

struct Counter {
    field_index: i16,
    static_index: i16,
    local_index: i16,
    argument_index: i16
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            field_index: 0,
            static_index: 0,
            local_index: 0,
            argument_index: 0
        }
    }

    pub fn index_by_kind(&self, kind: &SymbolKind) -> i16 {
        match kind {
            SymbolKind::Argument => self.argument_index,
            SymbolKind::Local => self.local_index,
            SymbolKind::Static => self.static_index,
            SymbolKind::Field => self.field_index
        }
    }

    pub fn increment_by_kind(&mut self, kind: &SymbolKind) {
        match kind {
            SymbolKind::Argument => self.argument_index += 1,
            SymbolKind::Local => self.local_index += 1,
            SymbolKind::Static => self.static_index += 1,
            SymbolKind::Field => self.field_index += 1
        }
    }
}

pub struct SymbolTable {
    counter: Counter,
    symbols: Vec<Symbol>
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            counter: Counter::new(),
            symbols: Vec::new()
        }
    }

    pub fn find_by(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|&s| s.var_name.as_str() == name)
    }

    pub fn field_vars_count(&self) -> i16 {
        self.symbols.iter().filter(|&s| match s.kind {
            SymbolKind::Field => true,
            _ => false
        }).count() as i16
    }

    pub fn push(&mut self, var_name: &str, var_type: Type, kind: SymbolKind) {
        let index = self.counter.index_by_kind(&kind);
        self.counter.increment_by_kind(&kind);
        let symbol = Symbol {
            var_name: var_name.to_string(),
            var_type,
            kind,
            index
        };
        self.symbols.push(symbol);
    }
}

pub struct CharSet(HashMap<char, i16>);

impl CharSet {
    pub fn new() -> Self {
        let mut set = HashMap::new();
        set.insert(' ', 32);
        set.insert('!', 33);
        set.insert('"', 34);
        set.insert('#', 35);
        set.insert('$', 36);
        set.insert('%', 37);
        set.insert('&', 38);
        set.insert('\'', 39);
        set.insert('(', 40);
        set.insert(')', 41);
        set.insert('*', 42);
        set.insert('+', 43);
        set.insert(',', 44);
        set.insert('-', 45);
        set.insert('.', 46);
        set.insert('/', 47);
        set.insert('0', 48);
        set.insert('1', 49);
        set.insert('2', 50);
        set.insert('3', 51);
        set.insert('4', 52);
        set.insert('5', 53);
        set.insert('6', 54);
        set.insert('7', 55);
        set.insert('8', 56);
        set.insert('9', 57);
        set.insert(':', 58);
        set.insert(';', 59);
        set.insert('<', 60);
        set.insert('=', 61);
        set.insert('>', 62);
        set.insert('?', 63);
        set.insert('@', 64);
        set.insert('A', 65);
        set.insert('B', 66);
        set.insert('C', 67);
        set.insert('D', 68);
        set.insert('E', 69);
        set.insert('F', 70);
        set.insert('G', 71);
        set.insert('H', 72);
        set.insert('I', 73);
        set.insert('J', 74);
        set.insert('K', 75);
        set.insert('L', 76);
        set.insert('M', 77);
        set.insert('N', 78);
        set.insert('O', 79);
        set.insert('P', 80);
        set.insert('Q', 81);
        set.insert('R', 82);
        set.insert('S', 83);
        set.insert('T', 84);
        set.insert('U', 85);
        set.insert('V', 86);
        set.insert('W', 87);
        set.insert('X', 88);
        set.insert('Y', 89);
        set.insert('Z', 90);
        set.insert('[', 91);
        set.insert('/', 92);
        set.insert(']', 93);
        set.insert('^', 94);
        set.insert('_', 95);
        set.insert('`', 96);
        set.insert('a', 97);
        set.insert('b', 98);
        set.insert('c', 99);
        set.insert('d', 100);
        set.insert('e', 101);
        set.insert('f', 102);
        set.insert('g', 103);
        set.insert('h', 104);
        set.insert('i', 105);
        set.insert('j', 106);
        set.insert('k', 107);
        set.insert('l', 108);
        set.insert('m', 109);
        set.insert('n', 110);
        set.insert('o', 111);
        set.insert('p', 112);
        set.insert('q', 113);
        set.insert('r', 114);
        set.insert('s', 115);
        set.insert('t', 116);
        set.insert('u', 117);
        set.insert('v', 118);
        set.insert('w', 119);
        set.insert('x', 120);
        set.insert('y', 121);
        set.insert('z', 122);
        set.insert('{', 123);
        set.insert('|', 124);
        set.insert('}', 125);
        set.insert('~', 126);
        set.insert('\u{007F}', 127); // DEL
        set.insert('\n', 128); // newLine
        set.insert('\u{0008}', 129); // backSpace
        set.insert('\u{2190}', 130); // leftArrow
        set.insert('\u{2191}', 131); // upArrow
        set.insert('\u{2192}', 132); // rightArrow
        set.insert('\u{2193}', 133); // downArrow
        CharSet(set)
    }

    pub fn decode(&self, char: char) -> i16 {
        *self.0.get(&char).unwrap()
    }
}

pub struct LabelGenerator {
    class_name: String,
    counter: i16
}

impl LabelGenerator {
    pub fn new(class_name: &str) -> Self {
        LabelGenerator {
            class_name: class_name.to_string(),
            counter: 0
        }
    }

    pub fn generate(&mut self) -> String {
        let label = format!("{}_{}", self.class_name.to_uppercase(), self.counter);
        self.counter += 1;
        label
    }
}