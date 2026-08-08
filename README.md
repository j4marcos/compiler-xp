Compiler-xp - experimental compiler in rust
===================

Grupo: 20230013339 — João Marcos Cunha Santos  
Repositorio: https://github.com/j4marcos/compiler-xp.git

Visão geral
-----------

Compilador lê um programa da linguagem **Func** a partir de um arquivo de entrada e gera código assembly x86-64 (sintaxe AT&T), com o fluxo:

1. **Análise léxica** (`src/lexical.rs`) — classifica tokens (números, operadores, identificadores, palavras-chave, blocos, etc.).
2. **Análise sintática** (`src/syntax.rs`) — monta a AST de expressões e a árvore de comandos do programa.
3. **Análise semântica** (`src/semantic.rs`) — valida declarações e uso de variáveis.
4. **Geração de código** (`src/generation.rs`) — emite assembly; o valor do `return` final fica em `%rax`, é impresso via runtime e o programa encerra.

O assembly é impresso no terminal e salvo em `output/target_code.s`.


### Linguagem Cmd

- Como variação da linguagem Cmd, a gramática foi expandida para incluir expressões booleanas 
and e or, operadores booleanos <= e >=, e operador %. E o return pode ser usado em qualquer 
bloco (obrigatório no bloco principal).

```
<programa> ::= <decl>* 'func' 'main' '(' <arglist>? ')' '{' <cmd>* <return> '}'
<ident>      ::= <letra><letra_digito>*
<decl> ::= <vardecl> | <fundecl>
<vardecl>     ::= 'var' <ident> '=' <exp> ';'
<fundecl> ::= 'func' <ident> '(' <arglist>? ')'
'{' <cmd>* <return> '}'
<arglist> ::= <ident> | <ident> ',' <arglist>
<cmd>      ::= <if> | <while> | <atrib> | <return> | <print> | <funcall> | <vardecl>
<print> ::= 'print' <exp> ';'
<if>       ::= 'if' <exp> '{' <cmd>* '}' 'else' '{' <cmd>* '}'
<while>    ::= 'while' <exp> '{' <cmd>* '}'
<atrib>    ::= <ident> '=' <exp> ';'
<return>   ::= 'return' <exp> ';'
<funcall>  ::= <ident> '(' <params>? ')'
<params>     ::= <exp> | <exp> ',' <params>

<exp>      ::= <exp_or>
<exp_or>   ::= <exp_and> (('or') <exp_and>)*
<exp_and>  ::= <exp_cmp> (('and') <exp_cmp>)*
<exp_cmp>  ::= <exp_a> (('<' | '>' | '==' | '<=' | '>=') <exp_a>)*
<exp_a>    ::= <exp_m> (('+' | '-') <exp_m>)*
<exp_m>    ::= <exp_u> (('*' | '/' | '%') <exp_u>)*
<exp_u>    ::= ('not') <exp_u> | <prim>
<prim>     ::= <num> | <ident> | '(' <exp> ')' | <funcall>
<num>      ::= <digito><digito>*
```

Precedência (maior → menor): `not` → `*` `/` `%` → `+` `-` → relacionais → `and` → `or`.



Estrutura
---------

### Token

Base da análise léxica. Cada token tem classe, lexema, coluna e linha (para erros).

```rust
pub enum TokenClass {
    Number,
    Attribution,
    Semicolon,
    Identifier,
    LeftParentheses,
    RightParentheses,
    OpenBlock,
    CloseBlock,
    SumOperator,
    SubOperator,
    DivOperator,
    MulOperator,
    ModOperator,
    EqualOperator,
    LessThanOperator,
    LessEqualOperator,
    GreaterThanOperator,
    GreaterEqualOperator,
    AndOperator,
    NotOperator,
    OrOperator,
    Space,
    NewLine,
    KeyWord,
}
```

Keywords: `if`, `else`, `while`, `return`, `and`, `or`, `not`.

### Expression / Command

```rust
enum Expression {
    NumberLiteral(i32),
    Identifier(String),
    UnaryOperation { operator: Operator, operand: Box<Expression> },
    BinOperation {
        left_value: Box<Expression>,
        operator: Operator,
        right_value: Box<Expression>,
    },
}

enum Operator {
    Sum, Sub, Div, Mul, Mod,
    Equal, LessThan, LessEqual, GreaterThan, GreaterEqual,
    And, Or, Not,
}

enum Command {
    If { condition, true_block, false_block },
    While { condition, block },
    Attribution { variable, expression },
    Return { expression },
}
```


Geração
-------

A geração percorre declarações e comandos e avalia expressões deixando o resultado em `%rax`:

1. literal / variável → `mov` para `%rax`
2. binária → avalia esquerda, `push %rax`, avalia direita, `pop %rbx`, aplica a operação
3. `if` / `while` → `cmp` + saltos (`jz` / `jmp`) com labels
4. `return` final → valor em `%rax`, depois `call imprime_num` e `call sair` (runtime em `templates/runtime.s`)


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
cargo run test/success/source_code_1
cargo run test/success/source_code_2
cargo run test/success/source_code_3
cargo run test/success/source_code_4
cargo run test/success/source_code_5
cargo run test/success/source_code_6
cargo run test/success/source_code_7
cargo run test/success/source_code_8
cargo run test/success/source_code_9
cargo run test/success/source_code_10
cargo run test/success/source_code_11
cargo run test/success/source_code_12
```

| Arquivo | Cobre |
|---------|--------|
| `1`–`3` | Aritmética e parênteses |
| `4` | Precedência `*` `/` `%` vs `+` `-` |
| `5`–`6` | Declarações e uso de variáveis |
| `7` | Comparações `==` `<` `>` `<=` `>=` |
| `8` | `and` / `or` / `not` |
| `9` | Módulo `%` |
| `10` | `if` / `else` |
| `11` | `while` |
| `12` | Programa combinando vários recursos |


Testes de erro
--------------

```sh
cargo run test/error/source_code_1
cargo run test/error/source_code_2
cargo run test/error/source_code_3
cargo run test/error/source_code_4
cargo run test/error/source_code_5
cargo run test/error/source_code_6
cargo run test/error/source_code_7
cargo run test/error/source_code_8
```

| Arquivo | Esperado |
|---------|----------|
| `1` | Erro sintático (expressão malformada) |
| `2` | Erro léxico (identificador/número inválido) |
| `3` | Erro sintático (arquivo vazio / sem bloco) |
| `4` | Erro sintático (bloco sem `return`) |
| `5` | Erro semântico (variável não declarada) |
| `6` | Erro semântico (uso antes da declaração) |
| `7` | Erro semântico (variável redeclarada) |
| `8` | Erro sintático (`if` sem `else`) |
