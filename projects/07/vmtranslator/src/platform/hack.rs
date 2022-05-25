use std::path::Path;

use crate::Translate;
use crate::parser::Command;
use crate::parser::Segment;
use crate::parser::Operator;
pub struct Hack {
    static_identifier: String,
    label_prefix: String,
    counter: i16,
    func_counter: i16
}

impl Hack {
    pub fn new(filename: &str) -> Self {
        let static_identifier = Path::new(filename).file_name().unwrap().to_str().unwrap();
        let static_identifier = static_identifier.strip_suffix(".vm").unwrap().to_string();
        let label_prefix = format!("{}_LABEL", static_identifier.to_uppercase());
        let counter = 0;
        let func_counter = 0;
        Hack {
            static_identifier,
            label_prefix,
            counter,
            func_counter
        }
    }

    pub fn bootstrap() -> String {
        format!("@256\nD=A\n@SP\nM=D\n{}", translate_call("Sys$ret", "Sys.init", 0))
    }

    pub fn end() -> String {
        "(END)\n@END\n0;JMP\n".to_string()
    }
}

const STACK_POP: &'static str = "\
@SP
AM=M-1
D=M";

const STACK_PUSH: &'static str = "\
@SP
A=M
M=D
@SP
M=M+1";

impl Translate for Hack {
    fn translate(&mut self, command: &Command) -> Option<String> {
        match command {
            Command::Push(segment, value) => {
                match segment {
                    Segment::Constant => {
                        Some(push_contant(*value))
                    },
                    Segment::Local => {
                        Some(push_segment("LCL", *value))
                    },
                    Segment::Argument => {
                        Some(push_segment("ARG", *value))
                    },
                    Segment::This => {
                        Some(push_segment("THIS", *value))
                    },
                    Segment::That => {
                        Some(push_segment("THAT", *value))
                    },
                    Segment::Static => {
                        let variable = format!("{}.{}", self.static_identifier, *value);
                        Some(push_static(&variable))
                    },
                    Segment::Temp => {
                        Some(push_temp(*value))
                    },
                    Segment::Pointer => {
                        Some(push_pointer(*value))
                    }
                }
            },
            Command::Pop(segment, value) => {
                match segment {
                    Segment::Local => {
                        Some(pop_segment("LCL", *value))
                    },
                    Segment::Argument => {
                        Some(pop_segment("ARG", *value))
                    },
                    Segment::This => {
                        Some(pop_segment("THIS", *value))
                    },
                    Segment::That => {
                        Some(pop_segment("THAT", *value))
                    },
                    Segment::Static => {
                        let variable = format!("{}.{}", self.static_identifier, *value);
                        Some(pop_static(&variable))
                    },
                    Segment::Temp => {
                        Some(pop_temp(*value))
                    },
                    Segment::Pointer => {
                        Some(pop_pointer(*value))
                    },
                    _ => None
                }
            },
            Command::Arithmetic(operator) => {
                match operator {
                    Operator::Add => {
                        Some(comp_x_and_y("M+D"))
                    },
                    Operator::Sub => {
                        Some(comp_x_and_y("M-D"))
                    },
                    Operator::And => {
                        Some(comp_x_and_y("D&M"))
                    },
                    Operator::Or => {
                        Some(comp_x_and_y("D|M"))
                    },
                    Operator::Neg => {
                        Some(comp_y("-M"))
                    },
                    Operator::Not => {
                        Some(comp_y("!M"))
                    },
                    Operator::Eq => {
                        let counter = self.counter;
                        self.counter += 1;
                        Some(comp_logic(counter, &self.label_prefix, "JEQ"))
                    },
                    Operator::Lt => {
                        let counter = self.counter;
                        self.counter += 1;
                        Some(comp_logic(counter, &self.label_prefix, "JLT"))
                    },
                    Operator::Gt => {
                        let counter = self.counter;
                        self.counter += 1;
                        Some(comp_logic(counter, &self.label_prefix, "JGT"))
                    }
                }
            },
            Command::Label(label) => {
                Some(format!("({})\n", label))
            },
            Command::GoTo(label) => {
                Some(format!("@{}\n0;JMP\n", label))
            },
            Command::IfGoTo(label) => {
                Some(format!("\
@SP
A=M-1
D=M
@SP
M=M-1
@{}
D;JNE
", label))
            },
            Command::Call(name, n_args) => {
                let return_label = format!("{}$ret.{}", self.static_identifier, self.func_counter);
                self.func_counter += 1;
                Some(translate_call(&return_label, name, *n_args))
            },
            Command::Function(name, n_vars) => {
                Some(translate_function(name, *n_vars))
            },
            Command::Return => {
                Some(translate_return())
            }
        }
    }
}

fn translate_call(return_label: &str, func_label: &str, n_args: i16) -> String {
    format!("\
@{}
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@5
D=D-A
@{}
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@{}
0;JMP
({})
", return_label, n_args, func_label, return_label)
}

fn translate_function(func_label: &str, n_vars: i16) -> String {
    let mut assembly = format!("({})\n", func_label);
    for _ in 0..n_vars {
        assembly.push_str("\
@SP
A=M
M=0
@SP
M=M+1
")
    }
    assembly
}

fn translate_return() -> String {
    format!("\
@LCL
D=M
@endframe
M=D
@5
A=D-A
D=M
@retaddr
M=D
@SP
AM=M-1
D=M
@ARG
A=M
M=D
@ARG
D=M+1
@SP
M=D
@endframe
AM=M-1
D=M
@THAT
M=D
@endframe
AM=M-1
D=M
@THIS
M=D
@endframe
AM=M-1
D=M
@ARG
M=D
@endframe
AM=M-1
D=M
@LCL
M=D
@retaddr
A=M
0;JMP
")
}

fn comp_x_and_y(expression: &str) -> String {
    format!("\
@SP
A=M-1
D=M
A=A-1
D={}
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1
", expression)
}

fn comp_y(expression: &str) -> String {
    format!("\
@SP
A=M-1
D={}
@SP
A=M-1
M=D
", expression)
}

fn comp_logic(counter: i16, label_prefix: &str, jump: &str) -> String {
    let label = format!("{}_{}", label_prefix, counter);
    format!("\
@SP
M=M-1
A=M
D=M
A=A-1
D=M-D
@{}
D;{}
@SP
A=M-1
M=0
@{}_END
0;JMP
({})
@SP
A=M-1
M=-1
({}_END)
", label, jump, label, label, label)
}

fn push_contant(value: i16) -> String {
    format!(
        "{}\n{}\n",
        load_constant(value),
        STACK_PUSH,
    )
}

fn push_segment(segment_base: &str, index: i16) -> String {
    format!(
        "{}\n{}\n",
        load_segment(segment_base, index),
        STACK_PUSH
    )
}

fn push_temp(index: i16) -> String {
    format!(
        "{}\n{}\n",
        load_temp(index),
        STACK_PUSH
    )
}

fn push_static(variable: &str) -> String {
    format!(
        "{}\n{}\n",
        load_static(&variable),
        STACK_PUSH
    )
}

fn push_pointer(value: i16) -> String {
    format!(
        "{}\n{}\n",
        load_pointer(value),
        STACK_PUSH
    )
}

fn pop_pointer(value: i16) -> String {
    let variable = match value {
        0 => "THIS",
        1 => "THAT",
        _ => panic!("Inavlue pointer index")
    };
    format!(
        "{}\n{}\n",
        STACK_POP,
        assign_variable(variable)
    )
}

fn pop_temp(index: i16) -> String {
    format!("\
{}
@R13
M=D
{}
@R13
A=M
M=D
", locate_temp(index), STACK_POP)
}

fn pop_segment(segment_base: &str, index: i16) -> String {
    format!("\
{}
@R13
M=D
{}
@R13
A=M
M=D
", locate_segment(segment_base, index), STACK_POP)
}

fn pop_static(variable: &str) -> String {
    format!(
        "{}\n{}\n",
        STACK_POP,
        assign_variable(&variable)
    )
}

fn load_pointer(index: i16) -> String {
    match index {
        0 => "@THIS\nD=M".to_string(),
        1 => "@THAT\nD=M".to_string(),
        _ => panic!("Invalid pointer index!")
    }
}

fn load_constant(value: i16) -> String {
    format!("\
@{}
D=A", value)
}

fn load_temp(index: i16) -> String {
    format!("\
@5
D=A
@{}
A=D+A
D=M", index)
}

fn load_segment(segment_id: &str, index: i16) -> String {
    format!("\
@{}
D=M
@{}
A=D+A
D=M", segment_id, index)
}

fn load_static(variable: &str) -> String {
    format!("\
@{}
D=M", variable)
}

fn locate_segment(segment_id: &str, index: i16) -> String {
    format!("\
@{}
D=M
@{}
D=D+A", segment_id, index)
}

fn locate_temp(index: i16) -> String {
    format!("\
@5
D=A
@{}
D=D+A", index)
}

fn assign_variable(variable: &str) -> String {
    format!("\
@{}
M=D", variable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label() {
        let command = Command::Label("LOOP".to_string());
        assert_eq!("\
(LOOP)
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn goto() {
        let command = Command::GoTo("LOOP".to_string());
        assert_eq!("\
@LOOP
0;JMP
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn if_goto() {
        let command = Command::IfGoTo("LOOP".to_string());
        assert_eq!("\
@SP
A=M-1
D=M
@SP
M=M-1
@LOOP
D;JNE
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn push_contant() {
        let command = Command::Push(Segment::Constant, 2);
        assert_eq!("\
@2
D=A
@SP
A=M
M=D
@SP
M=M+1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn push_argument() {
        let command = Command::Push(Segment::Argument, 0);
        assert_eq!("\
@ARG
D=M
@0
A=D+A
D=M
@SP
A=M
M=D
@SP
M=M+1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn push_static() {
        let command = Command::Push(Segment::Static, 3);
        assert_eq!("\
@Foo.3
D=M
@SP
A=M
M=D
@SP
M=M+1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn push_temp() {
        let command = Command::Push(Segment::Temp, 2);
        assert_eq!("\
@5
D=A
@2
A=D+A
D=M
@SP
A=M
M=D
@SP
M=M+1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn push_pointer() {
        let command = Command::Push(Segment::Pointer, 0);
        assert_eq!("\
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn pop_pointer() {
        let command = Command::Pop(Segment::Pointer, 1);
        assert_eq!("\
@SP
AM=M-1
D=M
@THAT
M=D
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn pop_temp() {
        let command = Command::Pop(Segment::Temp, 3);
        assert_eq!("\
@5
D=A
@3
D=D+A
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn pop_local() {
        let command = Command::Pop(Segment::Local, 3);
        assert_eq!("\
@LCL
D=M
@3
D=D+A
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn pop_static() {
        let command = Command::Pop(Segment::Static, 2);
        assert_eq!("\
@SP
AM=M-1
D=M
@Foo.2
M=D
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn add() {
        let command = Command::Arithmetic(Operator::Add);
        assert_eq!("\
@SP
A=M-1
D=M
A=A-1
D=M+D
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn sub() {
        let command = Command::Arithmetic(Operator::Sub);
        assert_eq!("\
@SP
A=M-1
D=M
A=A-1
D=M-D
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn neg() {
        let command = Command::Arithmetic(Operator::Neg);
        assert_eq!("\
@SP
A=M-1
D=-M
@SP
A=M-1
M=D
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn not() {
        let command = Command::Arithmetic(Operator::Not);
        assert_eq!("\
@SP
A=M-1
D=!M
@SP
A=M-1
M=D
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn and() {
        let command = Command::Arithmetic(Operator::And);
        assert_eq!("\
@SP
A=M-1
D=M
A=A-1
D=D&M
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn or() {
        let command = Command::Arithmetic(Operator::Or);
        assert_eq!("\
@SP
A=M-1
D=M
A=A-1
D=D|M
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn eq() {
        let command = Command::Arithmetic(Operator::Eq);
        assert_eq!("\
@SP
M=M-1
A=M
D=M
A=A-1
D=M-D
@FOO_LABEL_0
D;JEQ
@SP
A=M-1
M=0
@FOO_LABEL_0_END
0;JMP
(FOO_LABEL_0)
@SP
A=M-1
M=-1
(FOO_LABEL_0_END)
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn gt() {
        let command = Command::Arithmetic(Operator::Gt);
        assert_eq!("\
@SP
M=M-1
A=M
D=M
A=A-1
D=M-D
@FOO_LABEL_0
D;JGT
@SP
A=M-1
M=0
@FOO_LABEL_0_END
0;JMP
(FOO_LABEL_0)
@SP
A=M-1
M=-1
(FOO_LABEL_0_END)
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn lt() {
        let command = Command::Arithmetic(Operator::Lt);
        assert_eq!("\
@SP
M=M-1
A=M
D=M
A=A-1
D=M-D
@FOO_LABEL_0
D;JLT
@SP
A=M-1
M=0
@FOO_LABEL_0_END
0;JMP
(FOO_LABEL_0)
@SP
A=M-1
M=-1
(FOO_LABEL_0_END)
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        );
    }

    #[test]
    fn call_command() {
        let command = Command::Call("Foo.multiply".to_string(), 2);
        assert_eq!("\
@Foo$ret.0
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@5
D=D-A
@2
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Foo.multiply
0;JMP
(Foo$ret.0)
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        )
    }

    #[test]
    fn function_command() {
        let command = Command::Function("Foo.multiply".to_string(), 2);
        assert_eq!("\
(Foo.multiply)
@SP
A=M
M=0
@SP
M=M+1
@SP
A=M
M=0
@SP
M=M+1
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        )
    }

    #[test]
    fn return_command() {
        let command = Command::Return;
        assert_eq!("\
@LCL
D=M
@endframe
M=D
@5
A=D-A
D=M
@retaddr
M=D
@SP
M=M-1
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M+1
@SP
M=D
@endframe
AM=M-1
D=M
@THAT
M=D
@endframe
AM=M-1
D=M
@THIS
M=D
@endframe
AM=M-1
D=M
@ARG
M=D
@endframe
AM=M-1
D=M
@LCL
M=D
@endframe
A=M-1
A=M
0;JMP
".to_string(),
            Hack::new("Foo.vm").translate(&command).unwrap()
        )
    }
}
