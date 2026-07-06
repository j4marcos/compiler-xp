
.section .text
.globl _start
_start: 

push $7 
push $5 
pop %rbx 
pop %rax 
add %rbx, %rax 
push %rax 
push $3 
push $3 
pop %rbx 
pop %rax 
cqo 
idiv %rbx 
push %rax 
push $2 
pop %rbx 
pop %rax 
cqo 
idiv %rbx 
push %rax 
pop %rbx 
pop %rax 
add %rbx, %rax 
push %rax 
push $10 
push $8 
pop %rbx 
pop %rax 
imul %rbx, %rax 
push %rax 
push $2 
pop %rbx 
pop %rax 
sub %rbx, %rax 
push %rax 
pop %rbx 
pop %rax 
add %rbx, %rax 
push %rax 

pop %rax
call imprime_num
call sair
.include "runtime.s"
