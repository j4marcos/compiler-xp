.section .text
.globl _start
_start:

mov $25, %r8 # q
mov $05, %r9 # m
mov $20, %r10 # k
mov $26, %r11 # j
add $1, %r9
imul $13, %r9
mov %r9, %rax # parametro da divisão
cqo 
mov $5, %rcx
idiv %rcx # resultado em RAX
add %r8, %rax
add %r10, %rax
mov %rax, %r9
mov %r10, %rax
cqo
mov $4, %rcx
idiv %rcx
add %rax, %r9
mov %r11, %rax
cqo
mov $4, %rcx
idiv %rcx
add %rax, %r9
imul $2, %r11
sub %r11, %r9
mov %r9, %rax
mov $7, %rcx
idiv %rcx
mov %rdx, %rax
call imprime_num
call sair
.include "runtime.s"
