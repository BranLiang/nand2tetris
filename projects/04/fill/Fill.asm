// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// Put your code here.
@color
M=0
// Listen to keyboard input
(LOOP)
@KBD
D=M
// set the screen color to black if input found
@SETBLACK
D;JGT
// set the screen color to white if otherwise
@SETWHITE
D;JEQ

(SETBLACK)
@color
M=-1
@SYNC
0;JMP

(SETWHITE)
@color
M=0
@SYNC
0;JMP

(SYNC)
@row
M=0
@SCREEN
D=A
@address
M=D
@REFRESHROWS
0;JMP

(REFRESHROWS)
@col
M=0
@row
D=M
@256
D=D-A
@LOOP
D;JEQ
@row
M=M+1
@REFRESHROW
0;JMP

(REFRESHROW)
@col
D=M
@32
D=D-A
@REFRESHROWS
D;JEQ
@color
D=M
@address
A=M
M=D
@address
M=M+1
@col
M=M+1
@REFRESHROW
0;JMP
