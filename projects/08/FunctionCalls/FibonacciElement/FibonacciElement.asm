// Bootstrap
@256
D=A
@SP
M=D
@Sys$ret
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
@0
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Sys.init
0;JMP
(Sys$ret)
// function Main.fibonacci 0
(Main.fibonacci)
// push argument 0
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
// push constant 2
@2
D=A
@SP
A=M
M=D
@SP
M=M+1
// lt
@SP
M=M-1
A=M
D=M
A=A-1
D=M-D
@MAIN_LABEL_0
D;JLT
@SP
A=M-1
M=0
@MAIN_LABEL_0_END
0;JMP
(MAIN_LABEL_0)
@SP
A=M-1
M=-1
(MAIN_LABEL_0_END)
// if-goto if_true
@SP
A=M-1
D=M
@SP
M=M-1
@IF_TRUE
D;JNE
// goto if_false
@IF_FALSE
0;JMP
// label if_true
(IF_TRUE)
// push argument 0
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
// return
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
// label if_false
(IF_FALSE)
// push argument 0
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
// push constant 2
@2
D=A
@SP
A=M
M=D
@SP
M=M+1
// sub
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
// call Main.fibonacci 1
@Main$ret.0
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
@1
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Main.fibonacci
0;JMP
(Main$ret.0)
// push argument 0
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
// push constant 1
@1
D=A
@SP
A=M
M=D
@SP
M=M+1
// sub
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
// call Main.fibonacci 1
@Main$ret.1
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
@1
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Main.fibonacci
0;JMP
(Main$ret.1)
// add
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
// return
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
// function Sys.init 0
(Sys.init)
// push constant 4
@4
D=A
@SP
A=M
M=D
@SP
M=M+1
// call Main.fibonacci 1
@Sys$ret.0
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
@1
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Main.fibonacci
0;JMP
(Sys$ret.0)
// label while
(WHILE)
// goto while
@WHILE
0;JMP
// Program end
(END)
@END
0;JMP
