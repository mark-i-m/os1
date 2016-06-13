
	.global start
start:

	.extern main
	call main

	push %eax
loop:
	call exit
	jmp loop
