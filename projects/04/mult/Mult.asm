// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
//
// This program only needs to handle arguments that satisfy
// R0 >= 0, R1 >= 0, and R0*R1 < 32768.

// Put your code here.
// n=0
@n
M=0
// result=0
@r
M=0
(LOOP)
// if n == R1, goto STOP
@n
D=M
@R1
D=D-M
@STOP
D;JEQ
// result += R0
@R0
D=M
@r
M=D+M
// n += 1
@n
M=M+1
// goto LOOP
@LOOP
0;JMP
(STOP)
// R2 = result
@r
D=M
@R2
M=D
(END)
@END
0;JMP
