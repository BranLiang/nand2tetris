use crate::Translate;
use crate::parser::Command;
use crate::parser::Segment;
use crate::parser::Operator;
pub struct Hack {
    static_identifier: String,
    label_prefix: String,
    counter: i16
}

impl Hack {
    pub fn new(filename: &str) -> Self {
        let static_identifier = filename.strip_suffix(".vm").unwrap().to_string();
        let label_prefix = format!("{}_LABEL", static_identifier.to_uppercase());
        let counter = 0;
        Hack {
            static_identifier,
            label_prefix,
            counter
        }
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
                        Some(comp_logic(counter, &self.label_prefix, "JEQ", "JNE"))
                    },
                    Operator::Lt => {
                        let counter = self.counter;
                        self.counter += 1;
                        Some(comp_logic(counter, &self.label_prefix, "JLT", "JGE"))
                    },
                    Operator::Gt => {
                        let counter = self.counter;
                        self.counter += 1;
                        Some(comp_logic(counter, &self.label_prefix, "JGT", "JLE"))
                    }
                }
            },
            _ => None
        }
    }
}

fn comp_x_and_y(expression: &str) -> String {
    format!("\
@SP
A=M-1
D=M
A=M-1
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

fn comp_logic(counter: i16, label_prefix: &str, jump_a: &str, jump_b: &str) -> String {
    format!("\
@SP
A=M-1
D=M
A=M-1
D=D-M
@{}_{}T
D;{}
@{}_{}F
D;{}
({}_{}T)
M=-1
({}_{}F)
M=0
@SP
A=M-1
A=A-1
M=D
@SP
M=M-1
", label_prefix, counter, jump_a, label_prefix, counter, jump_b, label_prefix, counter, label_prefix, counter)
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
A=M-1
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
A=M-1
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
A=M-1
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
A=M-1
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
A=M-1
D=M
A=M-1
D=D-M
@FOO_LABEL_0T
D;JEQ
@FOO_LABEL_0F
D;JNE
(FOO_LABEL_0T)
M=-1
(FOO_LABEL_0F)
M=0
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
    fn gt() {
        let command = Command::Arithmetic(Operator::Gt);
        assert_eq!("\
@SP
A=M-1
D=M
A=M-1
D=D-M
@FOO_LABEL_0T
D;JGT
@FOO_LABEL_0F
D;JLE
(FOO_LABEL_0T)
M=-1
(FOO_LABEL_0F)
M=0
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
    fn lt() {
        let command = Command::Arithmetic(Operator::Lt);
        assert_eq!("\
@SP
A=M-1
D=M
A=M-1
D=D-M
@FOO_LABEL_0T
D;JLT
@FOO_LABEL_0F
D;JGE
(FOO_LABEL_0T)
M=-1
(FOO_LABEL_0F)
M=0
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
}
