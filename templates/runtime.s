  #
  # funcoes de apoio para o codigo compilado
  #

imprime_num:
  xor %r9, %r9            # rcx indice, r9 contagem
  mov $20, %rcx
  movb $10, buffer(%rcx)  # \n no final da string
  dec %rcx
  inc %r9

  mov $10, %r8
  or %rax, %rax
  jz printzero_L0
  jl mark_neg
  mov $0, %r10            # r10 flag p/ negativo
  jmp loop_L0

mark_neg:
  mov $1, %r10
  neg %rax

loop_L0:
  cqo
  idiv %r8
  addb $0x30, %dl
  movb %dl, buffer(%rcx)
  dec %rcx
  inc %r9
  or %rax, %rax
  jnz loop_L0
  test %r10, %r10
  jz print_L0
  movb $45, buffer(%rcx)
  dec %rcx
  jmp print_L0

printzero_L0:
  movb $0x30, buffer(%rcx)
  dec %rcx
  inc %r9

print_L0:
  mov $1, %rax            # sys_write
  mov $1, %rdi            # stdout
  mov $buffer, %rsi       # dados
  inc %rcx
  add %rcx, %rsi
  mov %r9, %rdx           # tamanho
  syscall
  ret

sair:
  mov $60, %rax     # sys_exit
  xor %rdi, %rdi    # codigo de saida (0)
  syscall

# --- heap (brk) + arrays com handle ---
# handle:  [ data_ptr ]
# bloco:   [ len ][ cap ][ data... ]
# convenção: args em %rdi/%rsi/%rdx, retorno em %rax

# alloc(size in %rdi) -> ptr in %rax
alloc:
  push %rbx
  mov %rdi, %rbx              # size
  cmpq $0, heap_break(%rip)
  jne alloc_have_break
  mov $12, %rax               # brk
  xor %rdi, %rdi
  syscall
  mov %rax, heap_break(%rip)
alloc_have_break:
  mov heap_break(%rip), %rax  # resultado = break atual
  mov %rax, %rdi
  add %rbx, %rdi              # novo break
  push %rax
  mov $12, %rax
  syscall
  mov %rax, heap_break(%rip)
  pop %rax
  pop %rbx
  ret

# realloc(old=%rdi, old_size=%rsi, new_size=%rdx) -> new ptr in %rax
realloc:
  push %rbx
  push %r12
  push %r13
  push %r14
  mov %rdi, %r12              # old
  mov %rsi, %r13              # old_size
  mov %rdx, %r14              # new_size
  mov %r14, %rdi
  call alloc
  mov %rax, %rbx              # new
  # copia min(old,new) bytes
  mov %r13, %rcx
  cmp %r14, %rcx
  jbe realloc_copy
  mov %r14, %rcx
realloc_copy:
  mov %r12, %rsi
  mov %rbx, %rdi
  cld
  rep movsb
  mov %rbx, %rax
  pop %r14
  pop %r13
  pop %r12
  pop %rbx
  ret

# array_new(n in %rdi) -> handle in %rax
array_new:
  push %rbx
  push %r12
  push %r13
  mov %rdi, %r12              # n = len
  mov %r12, %r13              # cap
  cmp $0, %r13
  jg array_new_cap_ok
  mov $1, %r13                # cap minimo 1
array_new_cap_ok:
  # handle = alloc(8)
  mov $8, %rdi
  call alloc
  mov %rax, %rbx              # handle
  # block_size = 16 + cap*8
  mov %r13, %rax
  imul $8, %rax
  add $16, %rax
  mov %rax, %rdi
  call alloc                  # data block
  mov %r12, (%rax)            # len
  mov %r13, 8(%rax)           # cap
  # zera data
  push %rax
  lea 16(%rax), %rdi
  mov %r13, %rcx
  xor %rax, %rax
  rep stosq
  pop %rax
  mov %rax, (%rbx)            # *handle = block
  mov %rbx, %rax              # return handle
  pop %r13
  pop %r12
  pop %rbx
  ret

# array_push(handle=%rdi, value=%rsi) -> handle in %rax
array_push:
  push %rbx
  push %r12
  push %r13
  push %r14
  mov %rdi, %r12              # handle
  mov %rsi, %r13              # value
  mov (%r12), %rbx            # data
  mov (%rbx), %r14            # len
  mov 8(%rbx), %rcx          # cap
  cmp %rcx, %r14
  jl array_push_store
  # cresce: new_cap = cap*2
  mov %rcx, %rax
  imul $2, %rax
  cmp $0, %rax
  jg array_push_newcap
  mov $1, %rax
array_push_newcap:
  mov %rax, %r8               # new_cap
  # old_size = 16 + cap*8
  mov %rcx, %rsi
  imul $8, %rsi
  add $16, %rsi
  # new_size = 16 + new_cap*8
  mov %r8, %rdx
  imul $8, %rdx
  add $16, %rdx
  mov %rbx, %rdi
  push %r8
  call realloc
  pop %r8
  mov %rax, %rbx
  mov %rbx, (%r12)            # *handle = new block
  mov %r8, 8(%rbx)            # cap = new_cap
array_push_store:
  # data[len] = value; len++
  mov %r14, %rax
  imul $8, %rax
  add $16, %rax
  add %rbx, %rax
  mov %r13, (%rax)
  inc %r14
  mov %r14, (%rbx)
  mov %r12, %rax              # return handle
  pop %r14
  pop %r13
  pop %r12
  pop %rbx
  ret

# array_pop(handle=%rdi) -> value in %rax
array_pop:
  push %rbx
  mov (%rdi), %rbx            # data
  mov (%rbx), %rcx            # len
  cmp $0, %rcx
  jg array_pop_ok
  xor %rax, %rax              # empty -> 0
  pop %rbx
  ret
array_pop_ok:
  dec %rcx
  mov %rcx, (%rbx)            # len--
  mov %rcx, %rax
  imul $8, %rax
  add $16, %rax
  add %rbx, %rax
  mov (%rax), %rax            # return data[old_len-1]
  pop %rbx
  ret

# parse_num(cstr in %rdi) -> i64 in %rax
parse_num:
  xor %rax, %rax
  xor %r8, %r8                # sign flag
  cmpb $'-', (%rdi)
  jne parse_num_loop
  mov $1, %r8
  inc %rdi
parse_num_loop:
  movzbq (%rdi), %rcx
  test %rcx, %rcx
  jz parse_num_done
  cmp $'0', %rcx
  jb parse_num_done
  cmp $'9', %rcx
  ja parse_num_done
  sub $'0', %rcx
  imul $10, %rax
  add %rcx, %rax
  inc %rdi
  jmp parse_num_loop
parse_num_done:
  test %r8, %r8
  jz parse_num_ret
  neg %rax
parse_num_ret:
  ret

# parse_text(cstr in %rdi) -> text handle in %rax
parse_text:
  push %rbx
  push %r12
  push %r13
  mov %rdi, %r12              # string
  xor %rcx, %rcx
parse_text_len:
  cmpb $0, (%rdi)
  je parse_text_len_done
  inc %rcx
  inc %rdi
  jmp parse_text_len
parse_text_len_done:
  mov %rcx, %r13              # len
  mov %r13, %rdi
  call array_new
  mov %rax, %rbx              # handle
  xor %rcx, %rcx
parse_text_copy:
  cmp %r13, %rcx
  jge parse_text_done
  movzbq (%r12, %rcx), %rdx
  mov (%rbx), %rax            # data block
  mov %rcx, %rdi
  imul $8, %rdi
  add $16, %rdi
  add %rax, %rdi
  mov %rdx, (%rdi)
  inc %rcx
  jmp parse_text_copy
parse_text_done:
  mov %rbx, %rax
  pop %r13
  pop %r12
  pop %rbx
  ret

  .section .bss
  .lcomm buffer, 21
  .lcomm heap_break, 8
  .lcomm __argv_base, 8
