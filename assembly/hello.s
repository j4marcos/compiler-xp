.section .data
hello:
 .ascii "Hello, World!\n"
.section .text
.globl _start
_start:
 mov $1, %rax # sys_write
 mov $1, %rdi # stdout
 mov $hello, %rsi # endereco do buffer
 mov $14, %rdx # numero de bytes
 syscall
 mov $60, %rax # sys_exit
 xor %rdi, %rdi # codigo de saida
 syscall