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
                let statements = StatementParser::new(self.tokenizer).collect();
                let body = SubroutineBody { var_decs, statements };
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

// StatementParser

struct StatementParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> StatementParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        StatementParser { tokenizer }
    }
}

impl<'a> Iterator for StatementParser<'a> {
    type Item=Statement;

    fn next(&mut self) -> Option<Self::Item> {
        if let Token::Keyword(v) = self.tokenizer.peek()? {
            match v.as_str() {
                "let" => {
                    // let
                    self.tokenizer.next();
                    // varName
                    let var_name = match self.tokenizer.next()? {
                        Token::Identifier(v) => VarName(v),
                        _ => return None
                    };
                    // [ expression ]
                    let index_expression = match self.tokenizer.peek()? {
                        Token::Symbol('[') => {
                            // '['
                            self.tokenizer.next();
                            // expression
                            let expression: Expression = Expression::parse(self.tokenizer)?;
                            // ']'
                            self.tokenizer.next();
                            Some(expression)
                        },
                        _ => None
                    };
                    // `=`
                    self.tokenizer.next()?;
                    // expression
                    let expression: Expression = Expression::parse(self.tokenizer)?;
                    // `;`
                    self.tokenizer.next()?;
                    let statement = LetStatement {
                        var_name,
                        index_expression,
                        expression
                    };
                    Some(Statement::Let(statement))
                },
                "if" => {
                    // if
                    self.tokenizer.next()?;
                    // `(`
                    self.tokenizer.next()?;
                    // expression
                    let expression = Expression::parse(self.tokenizer)?;
                    // `)`
                    self.tokenizer.next()?;
                    // `{`
                    self.tokenizer.next()?;
                    // if statements
                    let if_statements: Statements = StatementParser::new(self.tokenizer).collect();
                    // `}`
                    self.tokenizer.next()?;
                    // else statements
                    let else_statements = match self.tokenizer.peek()? {
                        Token::Keyword(v) if v.as_str() == "else" => {
                            // else
                            self.tokenizer.next();
                            // `{`
                            self.tokenizer.next();
                            // statements
                            let statements: Statements = StatementParser::new(self.tokenizer).collect();
                            // `}`
                            self.tokenizer.next();
                            Some(statements)
                        },
                        _ => None
                    };
                    let statement = IfStatement {
                        expression,
                        if_statements,
                        else_statements,
                    };
                    Some(Statement::If(Box::new(statement)))
                },
                "while" => {
                    // while
                    self.tokenizer.next();
                    // `(`
                    assert_symbol(&self.tokenizer.next()?, '(');
                    // expression
                    let expression = Expression::parse(self.tokenizer)?;
                    // `)`
                    assert_symbol(&self.tokenizer.next()?, ')');
                    // `{`
                    assert_symbol(&self.tokenizer.next()?, '{');
                    // statements
                    let statements = StatementParser::new(self.tokenizer).collect();
                    // `}`
                    assert_symbol(&self.tokenizer.next()?, '}');
                    let statement = WhileStatement {
                        expression,
                        statements,
                    };
                    Some(Statement::While(Box::new(statement)))
                },
                "do" => {
                    // do
                    self.tokenizer.next();
                    // subroutineCall
                    let subroutine_call = SubroutineCall::parse(self.tokenizer)?;
                    // `;`
                    assert_symbol(&self.tokenizer.next()?, ';');
                    Some(Statement::Do(subroutine_call))
                },
                "return" => {
                    // return
                    self.tokenizer.next();
                    // expression
                    let expression = Expression::parse(self.tokenizer);
                    // `;`
                    assert_symbol(&self.tokenizer.next()?, ';');
                    Some(Statement::Return(expression))
                },
                _ => None
            }
        } else {
            None
        }
    }
}

// ExtraExpressionParser

struct ExtraExpressionParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> ExtraExpressionParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        ExtraExpressionParser { tokenizer }
    }
}

impl<'a> Iterator for ExtraExpressionParser<'a> {
    type Item=Expression;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Symbol(',') => {
                // `,`
                self.tokenizer.next();
                Expression::parse(self.tokenizer)
            },
            _ => None
        }
    }
}

// ExtraOpTermsParser

struct ExtraOpTermsParser<'a> {
    tokenizer: &'a mut Peekable<Tokenizer>
}

impl<'a> ExtraOpTermsParser<'a> {
    pub fn new(tokenizer: &'a mut Peekable<Tokenizer>) -> Self {
        ExtraOpTermsParser { tokenizer }
    }
}

impl<'a> Iterator for ExtraOpTermsParser<'a> {
    type Item=OpTerm;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.peek()? {
            Token::Symbol('+') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Plus, term))
            },
            Token::Symbol('-') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Minus, term))
            },
            Token::Symbol('*') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Multiply, term))
            },
            Token::Symbol('/') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Divide, term))
            },
            Token::Symbol('&') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::And, term))
            },
            Token::Symbol('|') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Or, term))
            },
            Token::Symbol('<') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Lt, term))
            },
            Token::Symbol('>') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Gt, term))
            },
            Token::Symbol('=') => {
                // `unaryOp`
                self.tokenizer.next();
                // term
                let term = Term::parse(self.tokenizer)?;
                Some(OpTerm(Op::Eq, term))
            },
            _ => None
        }
    }
}

// Helpers

fn assert_keyword(token: &Token, keyword: &str) {
    match token {
        Token::Keyword(v) if v.as_str() == keyword => {},
        _ => panic!("{} doesn't match {:?}", keyword, token)
    }
}

fn assert_symbol(token: &Token, symbol: char) {
    match token {
        Token::Symbol(v) if *v == symbol => {},
        _ => panic!("{} doesn't match {:?}", symbol, token)
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
    Do(SubroutineCall),
    Return(Option<Expression>)
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

// Expressions

struct OpTerm(Op, Term);

struct Expression {
    term: Term,
    extra_op_terms: Vec<OpTerm>
}

impl Expression {
    pub fn parse_list(tokenizer: &mut Peekable<Tokenizer>) -> Vec<Expression> {
        let mut expression_list: Vec<Expression> = Vec::new();
        if let Some(expression) = Expression::parse(tokenizer) {
            expression_list.push(expression);
            for expression in ExtraExpressionParser::new(tokenizer) {
                expression_list.push(expression);
            }
        }
        expression_list
    }

    pub fn parse(tokenizer: &mut Peekable<Tokenizer>) -> Option<Self> {
        let term = Term::parse(tokenizer)?;
        let extra_op_terms = ExtraOpTermsParser::new(tokenizer).collect();
        Some(Expression {
            term,
            extra_op_terms,
        })
    }
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

impl Term {
    pub fn parse(tokenizer: &mut Peekable<Tokenizer>) -> Option<Self> {
        let token = (*tokenizer.peek()?).clone();
        match token {
            Token::Int(v) => {
                tokenizer.next();
                Some(Term::IntegerConstant(v))
            },
            Token::String(v) => {
                tokenizer.next();
                Some(Term::StringConstant(v))
            },
            Token::Keyword(v) if v.as_str() == "true" => {
                tokenizer.next();
                Some(Term::KeywordConstant(KeywordConstant::True))
            },
            Token::Keyword(v) if v.as_str() == "false" => {
                tokenizer.next();
                Some(Term::KeywordConstant(KeywordConstant::False))
            },
            Token::Keyword(v) if v.as_str() == "null" => {
                tokenizer.next();
                Some(Term::KeywordConstant(KeywordConstant::Null))
            },
            Token::Keyword(v) if v.as_str() == "this" => {
                tokenizer.next();
                Some(Term::KeywordConstant(KeywordConstant::This))
            },
            Token::Identifier(v) => {
                tokenizer.next();
                match tokenizer.peek() {
                    Some(Token::Symbol('[')) => {
                        // `[`
                        tokenizer.next();
                        // expression
                        let expression = Expression::parse(tokenizer)?;
                        // `]`
                        tokenizer.next();
                        Some(Term::IndexVar(v, Box::new(expression)))
                    },
                    Some(Token::Symbol('(')) => {
                        // `(`
                        tokenizer.next();
                        // expressionList
                        let expression_list = Expression::parse_list(tokenizer);
                        // `)`
                        tokenizer.next();
                        let subroutine_call = SubroutineCall {
                            caller: None,
                            subroutine_name: SubroutineName(v),
                            expression_list
                        };
                        Some(Term::Call(subroutine_call))
                    },
                    Some(Token::Symbol('.')) => {
                        // `.`
                        tokenizer.next();
                        // subroutineName
                        let subroutine_name = match tokenizer.next()? {
                            Token::Identifier(v) => SubroutineName(v),
                            _ => return None
                        };
                        // `(`
                        tokenizer.next();
                        // expressionList
                        let expression_list = Expression::parse_list(tokenizer);
                        // `)`
                        tokenizer.next();
                        let subroutine_call = SubroutineCall {
                            caller: Some(v),
                            subroutine_name,
                            expression_list
                        };
                        Some(Term::Call(subroutine_call))
                    },
                    _ => Some(Term::VarName(v))
                }
            },
            Token::Symbol('(') => {
                // `(`
                tokenizer.next();
                // expression
                let expression = Expression::parse(tokenizer)?;
                // `)`
                tokenizer.next();
                Some(Term::Expression(Box::new(expression)))
            },
            Token::Symbol('-') => {
                // unaryOp
                tokenizer.next();
                // term
                let term = Term::parse(tokenizer)?;
                Some(Term::WithUnary(UnaryOp::Negative, Box::new(term)))
            },
            Token::Symbol('~') => {
                // unaryOp
                tokenizer.next();
                // term
                let term = Term::parse(tokenizer)?;
                Some(Term::WithUnary(UnaryOp::Not, Box::new(term)))
            },
            _ => return None
        }
    }
}

struct SubroutineCall {
    caller: Option<String>,
    subroutine_name: SubroutineName,
    expression_list: Vec<Expression>,
}

impl SubroutineCall {
    pub fn parse(tokenizer: &mut Peekable<Tokenizer>) -> Option<Self> {
        match tokenizer.next()? {
            Token::Identifier(v) => {
                match tokenizer.peek()? {
                    Token::Symbol('(') => {
                        // `(`
                        assert_symbol(&tokenizer.next()?, '(');
                        // expressionList
                        let expression_list = Expression::parse_list(tokenizer);
                        // `)`
                        assert_symbol(&tokenizer.next()?, ')');
                        let subroutine_call = SubroutineCall {
                            caller: None,
                            subroutine_name: SubroutineName(v),
                            expression_list
                        };
                        Some(subroutine_call)
                    },
                    Token::Symbol('.') => {
                        // `.`
                        assert_symbol(&tokenizer.next()?, '.');
                        // subroutineName
                        let subroutine_name = match tokenizer.next()? {
                            Token::Identifier(v) => SubroutineName(v),
                            _ => return None
                        };
                        // `(`
                        assert_symbol(&tokenizer.next()?, '(');
                        // expressionList
                        let expression_list = Expression::parse_list(tokenizer);
                        // `)`
                        assert_symbol(&tokenizer.next()?, ')');
                        let subroutine_call = SubroutineCall {
                            caller: Some(v),
                            subroutine_name,
                            expression_list
                        };
                        Some(subroutine_call)
                    },
                    _ => None
                }
            },
            _ => None
        }
    }
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
    use core::panic;
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

    #[test]
    fn basic_expression_parser() {
        let mut tokenizer = fixture_tokenizer("a+b");
        let expression = Expression::parse(&mut tokenizer).unwrap();
        match expression {
            Expression { term: Term::VarName(a), extra_op_terms } if a == "a".to_string() => {
                let mut iter = extra_op_terms.iter();
                match iter.next().unwrap() {
                    OpTerm(Op::Plus, Term::VarName(v)) if v.as_str() == "b" => {},
                    _ => panic!("error parsing op term `+b`")
                }
                assert!(iter.next().is_none());
            },
            _ => panic!("error parsing expression `a+b`")
        }
    }

    #[test]
    fn complex_expression_parser() {
        let mut tokenizer = fixture_tokenizer("\
            -a - bob.age() / (get_max(size, 1) + alex[2])
        ");
        let expression = Expression::parse(&mut tokenizer).unwrap();
        match expression {
            Expression { term: Term::WithUnary(UnaryOp::Negative, t), extra_op_terms } => {
                match *t {
                    Term::VarName(v) => assert_eq!(v.as_str(), "a"),
                    _ => panic!("error parsing term `-a`")
                }
                let mut iter = extra_op_terms.into_iter();
                match iter.next().unwrap() {
                    OpTerm(
                        Op::Minus,
                        Term::Call(
                            SubroutineCall {
                                caller, 
                                subroutine_name: SubroutineName(v),
                                expression_list
                            }
                        )
                    ) => {
                        assert_eq!(caller, Some("bob".to_string()));
                        assert_eq!(v, "age".to_string());
                        assert!(expression_list.is_empty());
                    },
                    _ => panic!("error parsing op term `- bob.age`")
                }
                match iter.next().unwrap() {
                    OpTerm(
                        Op::Divide,
                        Term::Expression(expression)
                    ) => {
                        match *expression {
                            Expression {
                                term: Term::Call(
                                    SubroutineCall {
                                        caller,
                                        subroutine_name: SubroutineName(v),
                                        expression_list,
                                    }
                                ),
                                extra_op_terms,
                            } => {
                                assert_eq!(caller, None);
                                assert_eq!(v, "get_max".to_string());
                                let mut iter = expression_list.into_iter();
                                match iter.next().unwrap() {
                                    Expression { term: Term::VarName(v), extra_op_terms } => {
                                        assert_eq!(v, "size".to_string());
                                        assert!(extra_op_terms.is_empty());
                                    },
                                    _ => panic!()
                                }
                                match iter.next().unwrap() {
                                    Expression { term: Term::IntegerConstant(v), extra_op_terms } => {
                                        assert_eq!(v, 1);
                                        assert!(extra_op_terms.is_empty());
                                    },
                                    _ => panic!()
                                }
                                let mut iter = extra_op_terms.into_iter();
                                match iter.next().unwrap() {
                                    OpTerm(Op::Plus, Term::IndexVar(v, expression)) => {
                                        assert_eq!(v.as_str(), "alex");
                                        match *expression {
                                            Expression { term: Term::IntegerConstant(2), extra_op_terms } => {
                                                assert!(extra_op_terms.is_empty())
                                            },
                                            _ => panic!()
                                        }
                                    },
                                    _ => panic!()
                                }

                            },
                            _ => panic!()
                        }
                    },
                    _ => panic!("error parsing expression `/ (get_max(size, 1) + alex[2]`")
                }
                assert!(iter.next().is_none());
            },
            _ => panic!("error parsing complex expression")
        }
    }

    #[test]
    fn let_statement() {
        let mut tokenizer = fixture_tokenizer("\
            let a = 1;
            let b[1] = 2;
        ");
        let mut iter = StatementParser::new(&mut tokenizer);
        match iter.next().unwrap() {
            Statement::Let(
                LetStatement {
                    var_name: VarName(v),
                    index_expression: None,
                    expression: Expression {
                        term: Term::IntegerConstant(1),
                        extra_op_terms
                    }
                }
            ) => {
                assert_eq!(v.as_str(), "a");
                assert!(extra_op_terms.is_empty());
            },
            _ => panic!()
        }
        match iter.next().unwrap() {
            Statement::Let(
                LetStatement {
                    var_name: VarName(v),
                    index_expression: Some(
                        Expression {
                            term: Term::IntegerConstant(1),
                            extra_op_terms: extra_op_terms_1
                        }
                    ),
                    expression: Expression {
                        term: Term::IntegerConstant(2),
                        extra_op_terms
                    }
                }
            ) => {
                assert_eq!(v.as_str(), "b");
                assert!(extra_op_terms.is_empty());
                assert!(extra_op_terms_1.is_empty());
            },
            _ => panic!()
        }
    }

    #[test]
    fn if_statement() {
        let mut tokenizer = fixture_tokenizer("\
            if (true) {
                let a = 1;
            } else {
                let b = 2;
            }
        ");
        let mut iter = StatementParser::new(&mut tokenizer);
        match iter.next().unwrap() {
            Statement::If(statement) => {
                match *statement {
                    IfStatement {
                        expression: Expression {
                            term: Term::KeywordConstant(
                                KeywordConstant::True
                            ),
                            extra_op_terms,
                        },
                        if_statements,
                        else_statements: Some(else_statements),
                    } => {
                        assert!(extra_op_terms.is_empty());
                        assert_eq!(1, if_statements.len());
                        assert_eq!(1, else_statements.len());
                        match if_statements.first().unwrap() {
                            Statement::Let(_) => {},
                            _ => panic!()
                        }
                        match else_statements.first().unwrap() {
                            Statement::Let(_) => {},
                            _ => panic!()
                        }
                    },
                    _ => panic!()
                }
            },
            _ => panic!()
        }
    }

    #[test]
    fn while_statement() {
        let mut tokenizer = fixture_tokenizer("\
            while (true) {
                let a = 1;
            }
        ");
        let mut iter = StatementParser::new(&mut tokenizer);
        match iter.next().unwrap() {
            Statement::While(statement) => {
                match *statement {
                    WhileStatement {
                        expression: Expression {
                            term: Term::KeywordConstant(
                                KeywordConstant::True
                            ),
                            extra_op_terms
                        },
                        statements
                    } => {
                        assert!(extra_op_terms.is_empty());
                        assert_eq!(1, statements.len());
                    },
                    _ => panic!()
                }
            },
            _ => panic!()
        }
    }

    #[test]
    fn do_statement() {
        let mut tokenizer = fixture_tokenizer("\
            do get_max();
        ");
        let mut iter = StatementParser::new(&mut tokenizer);
        match iter.next().unwrap() {
            Statement::Do(
                SubroutineCall {
                    caller,
                    subroutine_name: SubroutineName(v),
                    expression_list,
                }
            ) => {
                assert_eq!(caller, None);
                assert_eq!(v.as_str(), "get_max");
                assert!(expression_list.is_empty());
            },
            _ => panic!()
        }
    }

    #[test]
    fn return_statement() {
        let mut tokenizer = fixture_tokenizer("\
            return 1;
        ");
        let mut iter = StatementParser::new(&mut tokenizer);
        match iter.next().unwrap() {
            Statement::Return(
                Some(
                    Expression {
                        term: Term::IntegerConstant(1),
                        extra_op_terms,
                    }
                )
            ) => {
                assert!(extra_op_terms.is_empty());
            },
            _ => panic!()
        }
    }
}