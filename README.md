Compiler-xp - experimental compiler in rust
===================

Grupo: 20230013339 — João Marcos Cunha Santos  
Repositorio: https://github.com/j4marcos/compiler-xp.git

Visão geral
-----------

Compilador lê um programa da linguagem **Func** a partir de um arquivo de entrada e gera código assembly x86-64 (sintaxe AT&T), com o fluxo:

1. **Análise léxica** (`src/lexical.rs`) — classifica tokens (números, operadores, identificadores, palavras-chave, blocos, etc.).
2. **Análise sintática** (`src/syntax.rs`) — monta a AST de expressões, declarações e funções.
3. **Análise semântica** (`src/semantic.rs`) — valida declarações, escopos e existência de `main` com `return` final.
4. **Geração de código** (`src/generation.rs`) — emite assembly; `_start` inicializa globais, chama `main`, imprime o valor em `%rax` via runtime e encerra.

O assembly é impresso no terminal e salvo em `output/target_code.s`.


### Linguagem Func

Variação da Cmd com `func`/`var`, expressões booleanas (`and`, `or`, `not`), operadores `<=`, `>=`, `%`, `+=`, `-=`, `*=`, `/=`, `++`, `print`, comentários de linha (`//` ignora o resto da linha), e funções com parâmetros e variáveis locais. O programa deve ter `func main()` e o último comando de `main` deve ser um `return`.

```
<programa> ::= <decl>*
<decl>       ::= <vardecl> | <fundecl>
<vardecl>    ::= 'var' <ident> '=' <exp> ';'
<fundecl>    ::= 'func' <ident> '(' <arglist>? ')' '{' <cmd>* '}'
<arglist>    ::= <ident> | <ident> ',' <arglist>
<cmd>        ::= <if> | <while> | <atrib> | <return> | <print> | <funcall> | <vardecl>
<print>      ::= 'print' <exp> ';'
<if>         ::= 'if' <exp> '{' <cmd>* '}' 'else' '{' <cmd>* '}'
<while>      ::= 'while' <exp> '{' <cmd>* '}'
<atrib>      ::= <ident> '=' <exp> ';'
             | <ident> '+=' <exp> ';'
             | <ident> '-=' <exp> ';'
             | <ident> '*=' <exp> ';'
             | <ident> '/=' <exp> ';'
             | <ident> '++' ';'
<return>     ::= 'return' <exp> ';'
<funcall>    ::= <ident> '(' <params>? ')'
<params>     ::= <exp> | <exp> ',' <params>

<exp>      ::= <exp_or>
<exp_or>   ::= <exp_and> (('or') <exp_and>)*
<exp_and>  ::= <exp_cmp> (('and') <exp_cmp>)*
<exp_cmp>  ::= <exp_a> (('<' | '>' | '==' | '<=' | '>=') <exp_a>)*
<exp_a>    ::= <exp_m> (('+' | '-') <exp_m>)*
<exp_m>    ::= <exp_u> (('*' | '/' | '%') <exp_u>)*
<exp_u>    ::= ('not') <exp_u> | <prim>
<prim>     ::= <num> | <ident> | '(' <exp> ')' | <funcall>
```

Precedência (maior → menor): `not` → `*` `/` `%` → `+` `-` → relacionais → `and` → `or`.

Exemplo:

```
func add(a, b) {
  return a + b;
}

func main() {
  return add(2, 3);
}
```


Estrutura
---------

### Token

Keywords: `if`, `else`, `while`, `return`, `print`, `and`, `or`, `not`, `func`, `var`, `main`.

### Expression / Command

```rust
enum Expression {
    NumberLiteral(i32),
    Identifier(String),
    UnaryOperation { operator, operand },
    BinOperation { left_value, operator, right_value },
    FunctionCall { name, parameters },
}

enum Command {
    If { condition, true_block, false_block },
    While { condition, block },
    Attribution { variable, expression },
    FunctionCall { name, parameters },
    Return { expression },
    Print { expression },
    Declaration { identifier }, // var ou func (func aninhada não é gerada)
}
```


Geração
-------

Expressões deixam o resultado em `%rax`.

1. literal / variável → `mov` para `%rax` (global por nome; local/param por offset de `%rbp`)
2. binária → esquerda, `push %rax`, direita, `pop %rbx`, operação
3. `if` / `while` → `cmp` + saltos com labels
4. **chamada de função** → empilha args na ordem inversa, `call`, limpa a pilha (`add $N, %rsp`); retorno em `%rax`
5. **corpo de função** → `push %rbp`, `sub $L*8, %rsp`, `mov %rsp, %rbp`; locais em `0(%rbp)…`; params em `(L+2)*8(%rbp)…`; no `return`: libera frame, `pop %rbp`, `ret`
6. `_start` → inicializa globais (`.bss`), `call main`, `call imprime_num`, `call sair` (`templates/runtime.s`)


Como executar
-------------

Requisito: [Rust](https://www.rust-lang.org/) instalado.

```sh
cargo run <CAMINHO_DO_ARQUIVO>
```

Montar e executar o assembly gerado:

```sh
as -o output/target_code.o output/target_code.s -Itemplates && ld -o output/target_code output/target_code.o && output/target_code
```


Testes de sucesso
-----------------

```sh
for i in $(seq 1 23); do
  cargo run test/success/source_code_$i
  as -o output/target_code.o output/target_code.s -Itemplates
  ld -o output/target_code output/target_code.o
  output/target_code
done
```




Testes de erro
--------------

```sh
for i in $(seq 1 10); do cargo run test/error/source_code_$i; done
```
