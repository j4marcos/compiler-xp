
.section .text
.globl _start
_start:
push $467 
push $3 
pop %rbx 
pop %rax 
cqo 
idiv %rbx 
push %rax 

pop %rax
call imprime_num
call sair
.include "runtime.s"
