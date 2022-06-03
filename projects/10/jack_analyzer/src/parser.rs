use std::fs::File;
use std::iter::Peekable;
use std::error::Error;
use std::io::Write;
use crate::tokenizer::Tokenizer;
use crate::tokenizer::Token;
use crate::utils::Padding;
use crate::utils::Symbol;
use crate::utils::SymbolTable;
use crate::utils::SymbolKind;
use crate::utils::CharSet;
use crate::utils::LabelGenerator;

pub struct XML;

impl XML {
    pub fn compile(file: File, output: &mut File) -> Result<(), Box<dyn Error>> {
        let mut tokenizer = Tokenizer::new(file)?.peekable();
        let parser = ClassParser::new(&mut tokenizer);
        let mut padding = Padding::new();
        for class in parser {
            println!("Parsing: {}", class.name.0);
            write!(output, "{}", class.to_xml(&mut padding))?;
        }
        Ok(())
    }

    pub fn symbol(symbol: char) -> String {
        format!("<symbol> {} </symbol>\n", symbol)
    }

    pub fn keyword(keywrod: &str) -> String {
        format!("<keyword> {} </keyword>\n", keywrod)
    }

    pub fn identifier(identifier: &str) -> String {
        format!("<identifier> {} </identifier>\n", identifier)
    }
}

pub struct VM {
    class_table: SymbolTable,
    subroutine_table: SymbolTable,
    label_generator: LabelGenerator,
    charset: CharSet,
    class_name: String
}

impl VM {
    pub fn new(class_name: &str) -> Self {
        VM {
            class_table: SymbolTable::new(),
            subroutine_table: SymbolTable::new(),
            label_generator: LabelGenerator::new(class_name),
            charset: CharSet::new(),
            class_name: class_name.to_string()
        }
    }

    pub fn compile(file: File, output: &mut File) -> Result<(), Box<dyn Error>> {
        let mut tokenizer = Tokenizer::new(file)?.peekable();
        let parser = ClassParser::new(&mut tokenizer);
        for class in parser {
            println!("Compiling: {}", class.name.0);
            let mut vm = VM::new(&class.name.0);
            write!(output, "{}", vm.compile_class(&class))?;
        }
        Ok(())
    }

    pub fn push(segment: &str, value: i16) -> String {
        format!("push {} {}\n", segment, value)
    }

    pub fn pop(segment: &str, index: i16) -> String {
        format!("pop {} {}\n", segment, index)
    }

    pub fn op(name: &str) -> String {
        format!("{}\n", name)
    }

    pub fn call(function_name: &str, n_args: i16) -> String {
        format!("call {} {}\n", function_name, n_args)
    }

    pub fn build(instructions: Vec<String>) -> String {
        let mut vm = String::new();
        for instruction in instructions.iter() {
            vm.push_str(instruction);
        }
        vm
    }

    pub fn label(label: &str) -> String {
        format!("label {}\n", label)
    }

    pub fn generate_label(&mut self) -> String {
        self.label_generator.generate()
    }

    pub fn goto(label: &str) -> String {
        format!("goto {}\n", label)
    }

    pub fn ifgoto(label: &str) -> String {
        format!("if-goto {}\n", label)
    }

    pub fn function(name: &str, n_vars: i16) -> String {
        format!("function {} {}\n", name, n_vars)
    }

    pub fn find_by(&self, name: &str) -> Option<&Symbol> {
        self.subroutine_table.find_by(name).or_else(|| self.class_table.find_by(name))
    }

    pub fn compile_string(&self, content: &str) -> String {
        let mut push_chars = String::new();
        for char in content.chars() {
            let char_number = self.charset.decode(char);
            push_chars.push_str(&VM::push("constant", char_number));
            push_chars.push_str(&VM::call("String.appendChar", 2));
        }
        VM::build(vec![
            VM::push("constant", content.len() as i16),
            VM::call("String.new", 1),
            push_chars
        ])
    }

    fn compile_class(&mut self, class: &Class) -> String {
        let mut instructions = String::new();
        // mapping class variables to the symbol table
        for var_dec in class.class_var_decs.iter() {
            self.class_table.push(
                &var_dec.var_name.0,
                var_dec.var_type.clone(),
                var_dec.dec_type.to_symbol_kind()
            );
            for extra_var_name in &var_dec.extra_var_names {
                self.class_table.push(
                    &extra_var_name.0,
                    var_dec.var_type.clone(),
                    var_dec.dec_type.to_symbol_kind()
                );
            }
        }
        // adding subroutine vm instructions
        for subroutine_dec in class.subroutine_decs.iter() {
            instructions.push_str(&self.compile_subroutine(&subroutine_dec))
        }
        instructions
    }

    fn compile_subroutine(&mut self, subroutine_dec: &SubroutineDec) -> String {
        self.subroutine_table = SymbolTable::new();
        // add method to the subroutine symbol table 
        if let SubroutineType::Method = subroutine_dec.subroutine_type {
            self.subroutine_table.push(
                "this",
                Type::ClassName(self.class_name.clone()),
                SymbolKind::Argument
            )
        }
        // add parameters to the subroutine symbol table
        for parameter in subroutine_dec.parameters.iter() {
            self.subroutine_table.push(
                &parameter.1.0,
                parameter.0.clone(),
                SymbolKind::Argument
            );
        }
        // handle local variables
        let mut n_vars = 0;
        for var_dec in subroutine_dec.body.var_decs.iter() {
            n_vars += 1;
            self.subroutine_table.push(
                &var_dec.var_name.0,
                var_dec.var_type.clone(),
                SymbolKind::Local
            );
            for extra_var_name in var_dec.extra_var_names.iter() {
                n_vars += 1;
                self.subroutine_table.push(
                    &extra_var_name.0,
                    var_dec.var_type.clone(),
                    SymbolKind::Local
                );
            }
        }

        let mut instructions = Vec::new();
        // function functionName nVars
        let function_name = format!("{}.{}", self.class_name, subroutine_dec.name.0);
        instructions.push(VM::function(&function_name, n_vars));

        match subroutine_dec.subroutine_type {
            SubroutineType::Constructor => {
                let field_vars_count = self.class_table.field_vars_count();
                instructions.push(VM::push("constant", field_vars_count));
                instructions.push(VM::call("Memory.alloc", 1));
                instructions.push(VM::pop("pointer", 0));
            },
            SubroutineType::Method => {
                // set THIS pointer to the value of argument 0
                instructions.push(VM::push("argument", 0));
                instructions.push(VM::pop("pointer", 0));
            },
            SubroutineType::Function => {}
        }
        // handle statements
        instructions.push(
            self.compile_statements(&subroutine_dec.body.statements, &subroutine_dec.return_type)
        );
        VM::build(instructions)
    }

    fn compile_statements(&mut self, statements: &Statements, return_type: &SubroutineReturnType) -> String {
        let mut instructions = Vec::new();
        for statement in statements.0.iter() {
            match statement {
                Statement::Do(subroutine_call) => {
                    instructions.push(self.compile_subroutine_call(subroutine_call));
                    instructions.push(VM::pop("temp", 0));
                },
                Statement::If(statement) => {
                    instructions.push(self.compile_if_statement(statement, return_type));
                },
                Statement::While(statement) => {
                    instructions.push(self.compile_while_statement(statement, return_type));
                },
                Statement::Let(statement) => {
                    instructions.push(self.compile_let_statement(statement));
                },
                Statement::Return(expression) => {
                    if let Some(expression) = expression {
                        instructions.push(self.compile_expression(expression));
                    } else if let SubroutineReturnType::Void = return_type {
                        instructions.push(VM::push("constant", 0));
                    }
                    instructions.push("return\n".to_string())
                }
            }
        }
        VM::build(instructions)
    }

    fn compile_subroutine_call(&self, subroutine_call: &SubroutineCall) -> String {
        let mut instructions = String::new();
        for expression in subroutine_call.expression_list.iter() {
            instructions.push_str(&self.compile_expression(expression));
        }
        match &subroutine_call.caller {
            None => {
                let command = format!("{}.{}", self.class_name, subroutine_call.subroutine_name.0);
                VM::build(vec![
                    VM::push("pointer", 0),
                    instructions,
                    VM::call(&command, subroutine_call.expression_list.len() as i16 + 1)
                ])
            },
            Some(caller) => {
                if let Some(symbol) = self.find_by(&caller) {
                    // handle method call
                    let segment = symbol.vm_memory_segment();
                    let index = symbol.index();
                    let command = format!("{}.{}", symbol.class_name(), subroutine_call.subroutine_name.0);
                    VM::build(vec![
                        VM::push(&segment, index),
                        instructions,
                        VM::call(&command, subroutine_call.expression_list.len() as i16 + 1)
                    ])
                } else {
                    // handle function calls and constructor calls
                    let command = format!("{}.{}", caller, subroutine_call.subroutine_name.0);
                    VM::build(vec![
                        instructions,
                        VM::call(&command, subroutine_call.expression_list.len() as i16)
                    ])
                }
            }
        }
    }

    fn compile_if_statement(&mut self, statement: &IfStatement, return_type: &SubroutineReturnType) -> String {
        let l1 = self.generate_label();
        let l2 = self.generate_label();

        let mut instructions = Vec::new();
        instructions.push(self.compile_expression(&statement.expression));
        instructions.push(VM::op("not"));
        instructions.push(VM::ifgoto(&l1));
        instructions.push(self.compile_statements(&statement.if_statements, return_type));
        instructions.push(VM::goto(&l2));
        instructions.push(VM::label(&l1));
        if let Some(statements) = &statement.else_statements {
            instructions.push(self.compile_statements(statements, return_type));
        }
        instructions.push(VM::label(&l2));
        VM::build(instructions)
    }

    fn compile_while_statement(&mut self, statement: &WhileStatement, return_type: &SubroutineReturnType) -> String {
        let l1 = self.generate_label();
        let l2 = self.generate_label();

        let mut instructions = Vec::new();
        instructions.push(VM::label(&l1));
        instructions.push(self.compile_expression(&statement.expression));
        instructions.push(VM::op("not"));
        instructions.push(VM::ifgoto(&l2));
        instructions.push(self.compile_statements(&statement.statements, return_type));
        instructions.push(VM::goto(&l1));
        instructions.push(VM::label(&l2));
        VM::build(instructions)
    }

    fn compile_let_statement(&self, statement: &LetStatement) -> String {
        let symbol = self.find_by(&statement.var_name.0).unwrap_or_else(|| {
            panic!("Var {} not found!", &statement.var_name.0);
        });
        if let Some(expression) = &statement.index_expression {
            // handle array index assignment
            VM::build(vec![
                VM::push(&symbol.vm_memory_segment(), symbol.index()),
                self.compile_expression(expression),
                VM::op("add"),
                self.compile_expression(&statement.expression),
                VM::pop("temp", 0),
                VM::pop("pointer", 1),
                VM::push("temp", 0),
                VM::pop("that", 0)
            ])
        } else {
            VM::build(vec![
                self.compile_expression(&statement.expression),
                VM::pop(&symbol.vm_memory_segment(), symbol.index())
            ])
        }
    }

    fn compile_expression(&self, expression: &Expression) -> String {
        let mut instructions = Vec::new();
        instructions.push(self.compile_term(&expression.term));
        for op_term in expression.extra_op_terms.iter() {
            instructions.push(self.compile_term(&op_term.1));
            instructions.push(self.compile_operation(&op_term.0));
        }
        VM::build(instructions)
    }

    fn compile_operation(&self, operation: &Op) -> String {
        match operation {
            Op::Plus => VM::op("add"),
            Op::Minus => VM::op("sub"),
            Op::Multiply => VM::call("Math.multiply", 2),
            Op::Divide => VM::call("Math.divide", 2),
            Op::And => VM::op("and"),
            Op::Or => VM::op("or"),
            Op::Lt => VM::op("lt"),
            Op::Gt => VM::op("gt"),
            Op::Eq => VM::op("eq")
        }
    }

    fn compile_unary_op(&self, unary_operation: &UnaryOp) -> String {
        match unary_operation {
            UnaryOp::Negative => VM::op("neg"),
            UnaryOp::Not => VM::op("not"),
        }
    }

    fn compile_term(&self, term: &Term) -> String {
        match term {
            Term::IntegerConstant(v) => VM::push("constant", *v),
            Term::VarName(v) => {
                let symbol = self.find_by(v).unwrap();
                VM::push(&symbol.vm_memory_segment(), symbol.index())
            },
            Term::KeywordConstant(v) => {
                match v {
                    KeywordConstant::Null => VM::push("constant", 0),
                    KeywordConstant::False => VM::push("constant", 0),
                    KeywordConstant::True => {
                        VM::build(vec![
                            VM::push("constant", 1),
                            VM::op("neg")
                        ])
                    },
                    KeywordConstant::This => VM::push("pointer", 0)
                }
            },
            Term::StringConstant(v) => self.compile_string(v),
            Term::Expression(expression) => self.compile_expression(expression),
            Term::Call(subroutine_call) => self.compile_subroutine_call(subroutine_call),
            Term::WithUnary(op, term) => {
                VM::build(vec![
                    self.compile_term(term),
                    self.compile_unary_op(op)
                ])
            },
            Term::IndexVar(var_name, expression) => {
                let symbol = self.find_by(var_name).unwrap();
                VM::build(vec![
                    // sets THAT
                    VM::push(&symbol.vm_memory_segment(), symbol.index()),
                    self.compile_expression(expression),
                    VM::op("add"),
                    VM::pop("pointer", 1),
                    VM::push("that", 0)
                ])
            }
        }
    }
}

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
                self.tokenizer.next();
                // className
                let name = match self.tokenizer.next()? {
                    Token::Identifier(v) => ClassName(v),
                    _ => return None
                };
                // '{'
                assert_symbol(&self.tokenizer.next()?, '{');
                // classVarDec*
                let class_var_decs = ClassVarDecParser::new(self.tokenizer).collect();
                // subroutineDec*
                let subroutine_decs = SubroutineDecParser::new(self.tokenizer).collect();
                // '}'
                assert_symbol(&self.tokenizer.next()?, '}');
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
                assert_symbol(&self.tokenizer.next()?, ';');
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
                assert_symbol(&self.tokenizer.next()?, '(');
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
                assert_symbol(&self.tokenizer.next()?, ')');
                // subroutineBody
                // `{`
                assert_symbol(&self.tokenizer.next()?, '{');
                // varDec*
                let var_decs = VarDecParser::new(self.tokenizer).collect();
                // statements
                let statements = Statements::parse(self.tokenizer);
                let body = SubroutineBody { var_decs, statements };
                // `}`
                assert_symbol(&self.tokenizer.next()?, '}');
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
                assert_symbol(&self.tokenizer.next()?, ';');
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
                            assert_symbol(&self.tokenizer.next()?, ']');
                            Some(expression)
                        },
                        _ => None
                    };
                    // `=`
                    assert_symbol(&self.tokenizer.next()?, '=');
                    // expression
                    let expression: Expression = Expression::parse(self.tokenizer)?;
                    // `;`
                    assert_symbol(&self.tokenizer.next()?, ';');
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
                    assert_symbol(&self.tokenizer.next()?, '(');
                    // expression
                    let expression = Expression::parse(self.tokenizer)?;
                    // `)`
                    assert_symbol(&self.tokenizer.next()?, ')');
                    // `{`
                    assert_symbol(&self.tokenizer.next()?, '{');
                    // if statements
                    let if_statements = Statements::parse(self.tokenizer);
                    // `}`
                    assert_symbol(&self.tokenizer.next()?, '}');
                    // else statements
                    let else_statements = match self.tokenizer.peek()? {
                        Token::Keyword(v) if v.as_str() == "else" => {
                            // else
                            self.tokenizer.next();
                            // `{`
                            assert_symbol(&self.tokenizer.next()?, '{');
                            // statements
                            let statements = Statements::parse(self.tokenizer);
                            // `}`
                            assert_symbol(&self.tokenizer.next()?, '}');
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
                    let statements = Statements::parse(self.tokenizer);
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

impl Class {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<class>\n");

        padding.increment();
        xml.push_str(&padding.to_spaces());
        xml.push_str("<keyword> class </keyword>\n");

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.name.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str("<symbol> { </symbol>\n");

        for class_var_dec in self.class_var_decs.iter() {
            xml.push_str(&class_var_dec.to_xml(padding));
        }

        for subroutine_dec in &self.subroutine_decs {
            xml.push_str(&subroutine_dec.to_xml(padding));
        }

        xml.push_str(&padding.to_spaces());
        xml.push_str("<symbol> } </symbol>\n");

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</class>\n");

        xml
    }
}

enum ClassVarDecType {
    Static,
    Field
}

impl ClassVarDecType {
    pub fn to_symbol_kind(&self) -> SymbolKind {
        match self {
            ClassVarDecType::Static => SymbolKind::Static,
            ClassVarDecType::Field => SymbolKind::Field
        }
    }

    pub fn new(v: &str) -> Option<Self> {
        match v {
            "static" => Some(Self::Static),
            "field" => Some(Self::Field),
            _ => None
        }
    }

    pub fn to_xml(&self) -> String {
        match self {
            ClassVarDecType::Field => "<keyword> field </keyword>\n".to_string(),
            ClassVarDecType::Static => "<keyword> static </keyword>\n".to_string()
        }
    }
}

struct ClassVarDec {
    dec_type: ClassVarDecType,
    var_type: Type,
    var_name: VarName,
    extra_var_names: Vec<VarName>
}

impl ClassVarDec {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();
        xml.push_str(&padding.to_spaces());
        xml.push_str("<classVarDec>\n");

        padding.increment();
        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.dec_type.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.var_type.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.var_name.to_xml());

        for var_name in &self.extra_var_names {
            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol(','));

            xml.push_str(&padding.to_spaces());
            xml.push_str(&var_name.to_xml());
        }

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(';'));

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</classVarDec>\n");

        xml
    }
}

#[derive(Clone)]
pub enum Type {
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

    pub fn to_xml(&self) -> String {
        match self {
            Type::Int => "<keyword> int </keyword>\n".to_string(),
            Type::Char => "<keyword> char </keyword>\n".to_string(),
            Type::Boolean => "<keyword> boolean </keyword>\n".to_string(),
            Type::ClassName(v) => format!("<identifier> {} </identifier>\n", v)
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

    pub fn to_xml(&self) -> String {
        match self {
            SubroutineType::Constructor => XML::keyword("constructor"),
            SubroutineType::Function => XML::keyword("function"),
            SubroutineType::Method => XML::keyword("method")
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

    pub fn to_xml(&self) -> String {
        match self {
            SubroutineReturnType::Void => XML::keyword("void"),
            SubroutineReturnType::General(t) => t.to_xml()
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

impl SubroutineDec {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();
        xml.push_str(&padding.to_spaces());
        xml.push_str("<subroutineDec>\n");

        padding.increment();
        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.subroutine_type.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.return_type.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.name.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('('));

        xml.push_str(&padding.to_spaces());
        xml.push_str("<parameterList>\n");

        padding.increment();
        if self.parameters.len() > 0 {
            let mut parameters = self.parameters.iter();
            let first_parameter = parameters.next().unwrap();
            
            xml.push_str(&first_parameter.to_xml(padding));
            for parameter in parameters {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol(','));

                xml.push_str(&parameter.to_xml(padding));
            }
        }
        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</parameterList>\n");

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(')'));

        xml.push_str(&self.body.to_xml(padding));

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</subroutineDec>\n");

        xml
    }
}

struct Parameter(Type, VarName);

impl Parameter {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();
        // Type
        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.0.to_xml());

        // varName
        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.1.to_xml());

        xml
    }
}

struct SubroutineBody {
    var_decs: Vec<VarDec>,
    statements: Statements
}

impl SubroutineBody {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<subroutineBody>\n");

        padding.increment();
        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('{'));

        for var_dec in self.var_decs.iter() {
            xml.push_str(&var_dec.to_xml(padding));
        }

        xml.push_str(&self.statements.to_xml(padding));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('}'));
        padding.decrement();

        xml.push_str(&padding.to_spaces());
        xml.push_str("</subroutineBody>\n");
        xml
    }
}

struct VarDec {
    var_type: Type,
    var_name: VarName,
    extra_var_names: Vec<VarName>
}

impl VarDec {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<varDec>\n");
        padding.increment();

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::keyword("var"));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.var_type.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.var_name.to_xml());

        for var_name in self.extra_var_names.iter() {
            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol(','));
            
            xml.push_str(&padding.to_spaces());
            xml.push_str(&var_name.to_xml());
        }

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(';'));

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</varDec>\n");

        xml
    }
}

struct ClassName(String);
impl ClassName {
    pub fn to_xml(&self) -> String {
        format!("<identifier> {} </identifier>\n", self.0)
    }
}

struct SubroutineName(String);
impl SubroutineName {
    pub fn to_xml(&self) -> String {
        format!("<identifier> {} </identifier>\n", self.0)
    }
}

struct VarName(String);
impl VarName {
    pub fn to_xml(&self) -> String {
        format!("<identifier> {} </identifier>\n", self.0)
    }
}

// Statements

struct Statements(Vec<Statement>);

impl Statements {
    pub fn parse(tokenizer: &mut Peekable<Tokenizer>) -> Self {
        Statements(
            StatementParser::new(tokenizer).collect()
        )
    }

    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        if self.0.len() > 0 {
            xml.push_str(&padding.to_spaces());
            xml.push_str("<statements>\n");
            padding.increment();

            for statement in self.0.iter() {
                xml.push_str(&statement.to_xml(padding));
            }

            padding.decrement();
            xml.push_str(&padding.to_spaces());
            xml.push_str("</statements>\n");
        }

        xml
    }
}

enum Statement {
    Let(LetStatement),
    If(Box<IfStatement>),
    While(Box<WhileStatement>),
    Do(SubroutineCall),
    Return(Option<Expression>)
}

impl Statement {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        match self {
            Statement::Let(statement) => {
                xml.push_str(&statement.to_xml(padding));
            },
            Statement::If(statement) => {
                xml.push_str(&statement.to_xml(padding));
            },
            Statement::While(statement) => {
                xml.push_str(&statement.to_xml(padding));
            },
            Statement::Do(subroutine_call) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str("<doStatement>\n");
                padding.increment();

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::keyword("do"));

                xml.push_str(&subroutine_call.to_xml(padding));

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol(';'));

                padding.decrement();
                xml.push_str(&padding.to_spaces());
                xml.push_str("</doStatement>\n");
            },
            Statement::Return(expression) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str("<returnStatement>\n");
                padding.increment();

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::keyword("return"));

                if let Some(expression) = expression {
                    xml.push_str(&expression.to_xml(padding));
                }

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol(';'));

                padding.decrement();
                xml.push_str(&padding.to_spaces());
                xml.push_str("</returnStatement>\n");
            }
        }

        xml
    }
}

struct LetStatement {
    var_name: VarName,
    index_expression: Option<Expression>,
    expression: Expression
}

impl LetStatement {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<letStatement>\n");
        padding.increment();

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::keyword("let"));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.var_name.to_xml());

        if let Some(expression) = &self.index_expression {
            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol('['));

            xml.push_str(&expression.to_xml(padding));

            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol(']'));
        }

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('='));

        xml.push_str(&self.expression.to_xml(padding));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(';'));

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</letStatement>\n");

        xml
    }
}

struct IfStatement {
    expression: Expression,
    if_statements: Statements,
    else_statements: Option<Statements>
}

impl IfStatement {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<ifStatement>\n");
        padding.increment();

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::keyword("if"));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('('));

        xml.push_str(&self.expression.to_xml(padding));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(')'));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('{'));

        xml.push_str(&self.if_statements.to_xml(padding));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('}'));

        if let Some(else_statements) = &self.else_statements {
            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::keyword("else"));

            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol('{'));

            xml.push_str(&else_statements.to_xml(padding));

            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol('}'));
        }

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</ifStatement>\n");

        xml
    }
}

struct WhileStatement {
    expression: Expression,
    statements: Statements
}

impl WhileStatement {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<whileStatement>\n");
        padding.increment();

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::keyword("while"));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('('));

        xml.push_str(&self.expression.to_xml(padding));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(')'));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('{'));

        xml.push_str(&self.statements.to_xml(padding));

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('}'));

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</whileStatement>\n");

        xml
    }
}

// Expressions

struct OpTerm(Op, Term);

impl OpTerm {
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.0.to_xml());
        xml.push_str(&self.1.to_xml(padding));

        xml
    }
}

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

    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<expression>\n");
        padding.increment();

        xml.push_str(&self.term.to_xml(padding));

        for op_term in self.extra_op_terms.iter() {
            xml.push_str(&op_term.to_xml(padding));
        }

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</expression>\n");

        xml
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
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        xml.push_str(&padding.to_spaces());
        xml.push_str("<term>\n");
        padding.increment();

        match self {
            Term::IntegerConstant(v) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&format!("<integerConstant> {} </integerConstant>\n", v));
            },
            Term::StringConstant(v) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&format!("<stringConstant> {} </stringConstant>\n", v));
            },
            Term::KeywordConstant(v) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&v.to_xml());
            },
            Term::VarName(v) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&format!("<identifier> {} </identifier>\n", v));
            },
            Term::IndexVar(v, expression) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&format!("<identifier> {} </identifier>\n", v));

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol('['));

                xml.push_str(&expression.to_xml(padding));

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol(']'));
            },
            Term::Call(subroutine_call) => {
                xml.push_str(&subroutine_call.to_xml(padding));
            },
            Term::Expression(expression) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol('('));

                xml.push_str(&expression.to_xml(padding));

                xml.push_str(&padding.to_spaces());
                xml.push_str(&XML::symbol(')'));
            },
            Term::WithUnary(op, term) => {
                xml.push_str(&padding.to_spaces());
                xml.push_str(&op.to_xml());
                
                xml.push_str(&term.to_xml(padding));
            }
        }

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</term>\n");

        xml
    }

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
                        assert_symbol(&tokenizer.next()?, ']');
                        Some(Term::IndexVar(v, Box::new(expression)))
                    },
                    Some(Token::Symbol('(')) => {
                        // `(`
                        tokenizer.next();
                        // expressionList
                        let expression_list = Expression::parse_list(tokenizer);
                        // `)`
                        assert_symbol(&tokenizer.next()?, ')');
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
                assert_symbol(&tokenizer.next()?, ')');
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
    pub fn to_xml(&self, padding: &mut Padding) -> String {
        let mut xml = String::new();

        if let Some(caller) = &self.caller {
            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::identifier(&caller));

            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol('.'));
        }

        xml.push_str(&padding.to_spaces());
        xml.push_str(&self.subroutine_name.to_xml());

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol('('));

        xml.push_str(&padding.to_spaces());
        xml.push_str("<expressionList>\n");
        padding.increment();

        let mut expressions = self.expression_list.iter();
        if let Some(expression) = expressions.next() {
            xml.push_str(&expression.to_xml(padding));
        }
        for expression in expressions {
            xml.push_str(&padding.to_spaces());
            xml.push_str(&XML::symbol(','));

            xml.push_str(&expression.to_xml(padding));
        }

        padding.decrement();
        xml.push_str(&padding.to_spaces());
        xml.push_str("</expressionList>\n");

        xml.push_str(&padding.to_spaces());
        xml.push_str(&XML::symbol(')'));

        xml
    }

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

impl KeywordConstant {
    pub fn to_xml(&self) -> String {
        match self {
            KeywordConstant::True => XML::keyword("true"),
            KeywordConstant::False => XML::keyword("false"),
            KeywordConstant::Null => XML::keyword("null"),
            KeywordConstant::This => XML::keyword("this")
        }
    }
}

enum UnaryOp {
    Negative,
    Not
}

impl UnaryOp {
    pub fn to_xml(&self) -> String {
        match self {
            UnaryOp::Negative => XML::symbol('-'),
            UnaryOp::Not => XML::symbol('~'),
        }
    }
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

impl Op {
    pub fn to_xml(&self) -> String {
        match self {
            Op::Plus => XML::symbol('+'),
            Op::Minus => XML::symbol('-'),
            Op::Multiply => XML::symbol('*'),
            Op::Divide => XML::symbol('/'),
            Op::And => "<symbol> &amp; </symbol>\n".to_string(),
            Op::Or => XML::symbol('|'),
            Op::Lt => "<symbol> &lt; </symbol>\n".to_string(),
            Op::Gt => "<symbol> &gt; </symbol>\n".to_string(),
            Op::Eq => XML::symbol('=')
        }
    }
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
            constructor People new(int age, String name) {
                var int a;
                let b = 1;
            }
            method int age() {}
        ");
        let mut parser = SubroutineDecParser::new(&mut tokenizer);

        match parser.next().unwrap() {
            SubroutineDec {
                subroutine_type: SubroutineType::Constructor,
                return_type: SubroutineReturnType::General(
                    Type::ClassName(a)
                ),
                name: SubroutineName(v),
                parameters,
                body: SubroutineBody {
                    var_decs,
                    statements: Statements(statements)
                }
            } => {
                assert_eq!(a.as_str(), "People");
                assert_eq!(v.as_str(), "new");
                let mut parameters = parameters.iter();
                match parameters.next().unwrap() {
                    Parameter(Type::Int, VarName(n)) if *n == "age".to_string() => {},
                    _ => panic!("error parsing parameter int age")
                }
                match parameters.next().unwrap() {
                    Parameter(Type::ClassName(c), VarName(n)) if *c == "String".to_string() && *n == "name".to_string() => {},
                    _ => panic!("error parsing parameter String name")
                }
                assert_eq!(1, var_decs.len());
                assert_eq!(1, statements.len());
            },
            _ => panic!()
        }

        match parser.next().unwrap() {
            SubroutineDec {
                subroutine_type: SubroutineType::Method,
                return_type: SubroutineReturnType::General(
                    Type::Int
                ),
                name: SubroutineName(v),
                parameters,
                body: SubroutineBody {
                    var_decs,
                    statements: Statements(statements)
                }
            } => {
                assert!(parameters.is_empty());
                assert_eq!(v.as_str(), "age");
                assert!(var_decs.is_empty());
                assert!(statements.is_empty());
            },
            _ => panic!()
        }
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
                        if_statements: Statements(if_statements),
                        else_statements: Some(
                            Statements(else_statements)
                        ),
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
                        statements: Statements(statements)
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