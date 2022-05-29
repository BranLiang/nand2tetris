use std::iter::Peekable;
use crate::tokenizer::Tokenizer;
use crate::tokenizer::Token;

// ClassParser

struct ClassParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> ClassParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        ClassParser { tokenizer }
    }
}

impl<'a> Iterator for ClassParser<'a> {
    type Item=Class;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Keyword(v) if *v == "class".to_string() => {
                // class keyword
                self.tokenizer.next()?;
                // className
                let name = match self.tokenizer.next()? {
                    Token::Identifier(v) => ClassName(v),
                    _ => return None
                };
                // '{'
                self.tokenizer.next()?;
                // classVarDec*
                let class_var_decs = ClassVarDecParser::new(self.tokenizer).collect();
                // subroutineDec*
                let subroutine_decs = SubroutineDecParser::new(self.tokenizer).collect();
                // '}'
                self.tokenizer.next()?;
                Some(Class { name, class_var_decs, subroutine_decs })
            },
            _ => None
        }
    }
}

// ClassVarDecParser

struct ClassVarDecParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> ClassVarDecParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        ClassVarDecParser { tokenizer }
    }
}

impl<'a> Iterator for ClassVarDecParser<'a> {
    type Item=ClassVarDec;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Keyword(v)  => {
                // static | field
                let dec_type = ClassVarDecType::new(&v)?;
                self.tokenizer.next();
                // Type
                let token = self.tokenizer.next()?;
                let var_type = Type::new(&token)?;
                // var_name
                let var_name = match self.tokenizer.next()? {
                    Token::Identifier(v) => VarName(v),
                    _ => return None
                };
                // exta_var_names
                let extra_var_names = ExtraVarNameParser::new(self.tokenizer).collect();
                // `;`
                self.tokenizer.next()?;
                Some(ClassVarDec { dec_type, var_type, var_name, extra_var_names })
            },
            _ => None
        }
    }
}

// SubroutineDecParser

struct SubroutineDecParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> SubroutineDecParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        SubroutineDecParser { tokenizer }
    }
}

impl<'a> Iterator for SubroutineDecParser<'a> {
    type Item=SubroutineDec;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Keyword(v) => {
                // constructor | function | method
                let subroutine_type = SubroutineType::new(&v)?;
                self.tokenizer.next();
                // return type
                let token = self.tokenizer.next()?;
                let return_type = SubroutineReturnType::new(&token)?;
                // name
                let name = match self.tokenizer.next()? {
                    Token::Identifier(v) => SubroutineName(v),
                    _ => return None
                };
                // `(`
                self.tokenizer.next();
                // Parameter list
                let mut parameters = Vec::new();
                match self.tokenizer.peek()? {
                    Token::Symbol(')') => {},
                    _ => {
                        // First parameter
                        let token = self.tokenizer.next()?;
                        let parameter_type = Type::new(&token)?;
                        let var_name = match self.tokenizer.next()? {
                            Token::Identifier(v) => VarName(v),
                            _ => return None
                        };
                        parameters.push(Parameter(parameter_type, var_name));
                        // Extra parameters
                        for paramter in ExtraParameterParser::new(self.tokenizer) {
                            parameters.push(paramter);
                        }
                    }
                }
                // `)`
                self.tokenizer.next()?;
                // subroutineBody
                // `{`
                self.tokenizer.next();
                // varDec*
                let var_decs = VarDecParser::new(self.tokenizer).collect();
                // statements
                // TODO
                let body = SubroutineBody { var_decs, statements: vec![] };
                // `}`
                self.tokenizer.next();
                Some(SubroutineDec {
                    subroutine_type,
                    return_type,
                    name,
                    parameters,
                    body
                })
            },
            _ => None
        }
    }
}

// VarDecParser

struct VarDecParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> VarDecParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        VarDecParser { tokenizer }
    }
}

impl<'a> Iterator for VarDecParser<'a> {
    type Item=VarDec;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Keyword(v) if *v == "var".to_string() => {
                // var
                self.tokenizer.next();
                // type
                let token = self.tokenizer.next()?;
                let var_type = Type::new(&token)?;
                // varName
                let var_name = match self.tokenizer.next()? {
                    Token::Identifier(v) => VarName(v),
                    _ => return None
                };
                // extra var names
                let extra_var_names = ExtraVarNameParser::new(self.tokenizer).collect();
                // `;`
                self.tokenizer.next()?;
                Some(VarDec { var_type, var_name, extra_var_names })
            },
            _ => None
        }
    }
}

// ExtraVarNameParser

struct ExtraVarNameParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> ExtraVarNameParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        ExtraVarNameParser { tokenizer }
    }
}

impl<'a> Iterator for ExtraVarNameParser<'a> {
    type Item=VarName;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Symbol(',') => {
                // `,`
                self.tokenizer.next();
                // varName
                match self.tokenizer.next()? {
                    Token::Identifier(v) => Some(VarName(v)),
                    _ => None
                }
            },
            _ => None
        }
    }
}

// Parameter parser
struct ExtraParameterParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> ExtraParameterParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        ExtraParameterParser { tokenizer }
    }
}

impl<'a> Iterator for ExtraParameterParser<'a> {
    type Item=Parameter;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Symbol(',') => {
                // `,`
                self.tokenizer.next();
                // type varName
                let token = self.tokenizer.next()?;
                let var_type = Type::new(&token)?;
                match self.tokenizer.next()? {
                    Token::Identifier(v) => {
                        Some(Parameter(var_type, VarName(v)))
                    },
                    _ => None
                }
            },
            _ => None
        }
        
    }
}

// Program structure

struct Class {
    name: ClassName,
    class_var_decs: Vec<ClassVarDec>,
    subroutine_decs: Vec<SubroutineDec>
}

enum ClassVarDecType {
    Static,
    Field
}

impl ClassVarDecType {
    pub fn new(v: &str) -> Option<Self> {
        match v {
            "static" => Some(Self::Static),
            "field" => Some(Self::Field),
            _ => None
        }
    }
}

struct ClassVarDec {
    dec_type: ClassVarDecType,
    var_type: Type,
    var_name: VarName,
    extra_var_names: Vec<VarName>
}
enum Type {
    Int,
    Char,
    Boolean,
    ClassName(String)
}

impl Type {
    pub fn new(token: &Token) -> Option<Self> {
        match token {
            Token::Keyword(v) if *v == "int".to_string() => Some(Type::Int),
            Token::Keyword(v) if *v == "char".to_string() => Some(Type::Char),
            Token::Keyword(v) if *v == "boolean".to_string() => Some(Type::Boolean),
            Token::Identifier(v) => Some(Type::ClassName((*v).clone())),
            _ => None
        }
    }
}

enum SubroutineType {
    Constructor,
    Function,
    Method
}

impl SubroutineType {
    pub fn new(v: &str) -> Option<Self> {
        match v {
            "constructor" => Some(Self::Constructor),
            "function" => Some(Self::Function),
            "method" => Some(Self::Method),
            _ => None
        }
    }
}

enum SubroutineReturnType {
    Void,
    General(Type)
}

impl SubroutineReturnType {
    pub fn new(token: &Token) -> Option<Self> {
        match token {
            Token::Keyword(v) if *v == "void".to_string() => Some(Self::Void),
            _ => {
                let kind = Type::new(token)?;
                Some(Self::General(kind))
            }
        }
    }
}

struct SubroutineDec {
    subroutine_type: SubroutineType,
    return_type: SubroutineReturnType,
    name: SubroutineName,
    parameters: Vec<Parameter>,
    body: SubroutineBody
}

struct Parameter(Type, VarName);

struct SubroutineBody {
    var_decs: Vec<VarDec>,
    statements: Vec<Statement>
}

struct VarDec {
    var_type: Type,
    var_name: VarName,
    extra_var_names: Vec<VarName>
}

struct ClassName(String);
struct SubroutineName(String);
struct VarName(String);

// Statements

type Statements = Vec<Statement>;

enum Statement {
    Let(LetStatement),
    If(Box<IfStatement>),
    While(Box<WhileStatement>),
    Do(DoStatement),
    Return(ReturnStatement)
}

struct LetStatement {
    var_name: VarName,
    index_expression: Option<Expression>,
    expression: Expression
}

struct IfStatement {
    expression: Expression,
    if_statements: Statements,
    else_statements: Option<Statements>
}

struct WhileStatement {
    expression: Expression,
    statements: Statements
}

struct DoStatement(SubroutineCall);

struct ReturnStatement(Option<Expression>);

// Expressions

struct OpTerm(Op, Term);

struct Expression {
    term: Term,
    extra_op_terms: Vec<OpTerm>
}

enum Term {
    IntegerConstant(i16),
    StringConstant(String),
    KeywordConstant(KeywordConstant),
    VarName(String),
    IndexVar(String, Box<Expression>),
    Call(SubroutineCall),
    Expression(Box<Expression>),
    WithUnary(UnaryOp, Box<Term>)
}

struct SubroutineCall {
    caller: Option<String>,
    subroutine_name: SubroutineName,
    expression_list: Vec<Expression>,
}

enum KeywordConstant {
    True,
    False,
    Null,
    This
}

enum UnaryOp {
    Negative,
    Not
}

enum Op {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Lt,
    Gt,
    Eq
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempfile;
    use std::io::SeekFrom;
    use std::io::prelude::*;

    fn fixture_tokenizer(content: &str) -> Peekable<Tokenizer> {
        let mut file = tempfile().unwrap();
        for line in content.lines() {
            writeln!(file, "{}", line).unwrap();
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        Tokenizer::new(file).unwrap().peekable()
    }

    #[test]
    fn extra_var_names_parser() {
        let mut tokenizer = fixture_tokenizer(", hello, world");
        let mut parser = ExtraVarNameParser::new(&mut tokenizer);
        match parser.next() {
            Some(VarName(v)) if v == "hello".to_string() => {},
            _ => panic!("error parsing var `hello`")
        }
        match parser.next() {
            Some(VarName(v)) if v == "world".to_string() => {},
            _ => panic!("error parsing var `world`")
        }
        assert!(parser.next().is_none());
    }

    #[test]
    fn extra_paramters_parser() {
        let mut tokenizer = fixture_tokenizer(", int a, boolean isTrue, People bran");
        let mut parser = ExtraParameterParser::new(&mut tokenizer);
        match parser.next() {
            Some(Parameter(Type::Int, VarName(v))) if v == "a".to_string() => {},
            _ => panic!("error parsing int parameter a")
        }
        match parser.next() {
            Some(Parameter(Type::Boolean, VarName(v))) if v == "isTrue".to_string() => {},
            _ => panic!("error parsing boolean parameter isTrue")
        }
        match parser.next() {
            Some(Parameter(Type::ClassName(c), VarName(v))) if c == "People" && v == "bran".to_string() => {},
            _ => panic!("error parsing classname parameter bran")
        }
        assert!(parser.next().is_none());
    }

    #[test]
    fn class_var_dec_parser() {
        let mut tokenizer = fixture_tokenizer("\
            static int a, b;
            field boolean c, d;
        ");
        let mut parser = ClassVarDecParser::new(&mut tokenizer);

        let ClassVarDec {
            dec_type,
            var_type,
            var_name,
            extra_var_names
        } = parser.next().unwrap();
        match dec_type {
            ClassVarDecType::Static => {},
            _ => panic!("error parsing dec_type")
        }
        match var_type {
            Type::Int => {},
            _ => panic!("error parsing var_type")
        }
        match var_name {
            VarName(v) if v == "a".to_string() => {},
            _ => panic!("error parsing int a")
        }
        match extra_var_names.first().unwrap() {
            VarName(v) if *v == "b".to_string() => {},
            _ => panic!("error parsing int b")
        }

        let ClassVarDec {
            dec_type,
            var_type,
            var_name,
            extra_var_names
        } = parser.next().unwrap();
        match dec_type {
            ClassVarDecType::Field => {},
            _ => panic!("error parsing dec_type")
        }
        match var_type {
            Type::Boolean => {},
            _ => panic!("error parsing var_type")
        }
        match var_name {
            VarName(v) if v == "c".to_string() => {},
            _ => panic!("error parsing int c")
        }
        match extra_var_names.first().unwrap() {
            VarName(v) if *v == "d".to_string() => {},
            _ => panic!("error parsing int d")
        }

        assert!(parser.next().is_none());
    }

    #[test]
    fn subroutine_dec_parser() {
        let mut tokenizer = fixture_tokenizer("\
            constructor People new(int age, String name) {}
            method int age() {}
        ");
        let mut parser = SubroutineDecParser::new(&mut tokenizer);

        let SubroutineDec {
            subroutine_type,
            return_type,
            name,
            parameters,
            body
        } = parser.next().unwrap();
        match subroutine_type {
            SubroutineType::Constructor => {},
            _ => panic!("error parsing subroutine type")
        }
        match return_type {
            SubroutineReturnType::General(v) => {
                match v {
                    Type::ClassName(c) if c == "People".to_string() => {},
                    _ => panic!("error parsing return type 1")
                }
            },
            _ => panic!("error parsing return type 2")
        }
        match name {
            SubroutineName(v) if v == "new".to_string() => {},
            _ => panic!("error parsing subroutine name")
        }
        let mut parameters = parameters.iter();
        match parameters.next().unwrap() {
            Parameter(Type::Int, VarName(n)) if *n == "age".to_string() => {},
            _ => panic!("error parsing parameter int age")
        }
        match parameters.next().unwrap() {
            Parameter(Type::ClassName(c), VarName(n)) if *c == "String".to_string() && *n == "name".to_string() => {},
            _ => panic!("error parsing parameter String name")
        }

        // let SubroutineDec {
        //     subroutine_type,
        //     return_type,
        //     name,
        //     parameters,
        //     body
        // } = parser.next().unwrap();
        // match subroutine_type {
        //     SubroutineType::Method => {},
        //     _ => panic!("error parsing subroutine type")
        // }
        // match return_type {
        //     SubroutineReturnType::General(v) => {
        //         match v {
        //             Type::Int => {},
        //             _ => panic!("error parsing return type 1")
        //         }
        //     },
        //     _ => panic!("error parsing return type 2")
        // }
        // match name {
        //     SubroutineName(v) if v == "age".to_string() => {},
        //     _ => panic!("error parsing subroutine name")
        // }
        // assert!(parameters.is_empty());
    }

    #[test]
    fn var_dec_parser() {
        let mut tokenizer = fixture_tokenizer("\
            var int age, weight, height;
            var String name;
        ");
        let mut parser = VarDecParser::new(&mut tokenizer);

        let VarDec {
            var_type,
            var_name,
            extra_var_names
        } = parser.next().unwrap();
        match var_type {
            Type::Int => {},
            _ => panic!("error parsing var type")
        }
        match var_name {
            VarName(v) if v == "age".to_string() => {},
            _ => panic!("error parsing var_name")
        }
        let mut extra_var_names = extra_var_names.iter();
        match extra_var_names.next().unwrap() {
            VarName(v) if *v == "weight".to_string() => {},
            _ => panic!("errpr parsing weight")
        }
        match extra_var_names.next().unwrap() {
            VarName(v) if *v == "height".to_string() => {},
            _ => panic!("errpr parsing weight")
        }
        assert!(extra_var_names.next().is_none());

        let VarDec {
            var_type,
            var_name,
            extra_var_names
        } = parser.next().unwrap();
        match var_type {
            Type::ClassName(v) if v == "String".to_string() => {},
            _ => panic!("error parsing var type")
        }
        match var_name {
            VarName(v) if v == "name".to_string() => {},
            _ => panic!("error parsing var_name")
        }
        assert!(extra_var_names.is_empty());
    }
}