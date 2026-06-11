
Compiler-xp - experimental compiler in rust
===================

Grupo: 20230013339 — João Marcos Cunha Santos
Repositorio: https://github.com/j4marcos/compiler-xp.git

Visão geral
-----------

Compilador lê a expressão aritmética de um arquivo de entrada e gera o código assembly com o fluxo de 3 etapas:

1. **Análise léxica** (`src/lexical.rs`) — identifica e classifica tokens: números, operadores, parênteses e espaços.
2. **Análise sintática** (`src/syntax.rs`) — monta a **árvore de expressão (AST)** a partir da lista de tokens.

3. **Geração de código** (`src/generation.rs`) — gera o código assembly a partir da árvore de expressão.

imprime o resultado no terminal e salva em arquivo assembly em `output/target_code.s`.

Estrutura
---------

### token

O Token é a base da analise léxica. Ele é representado por um enum com as classes. Cada token possui um lexema, uma coluna e uma linha que rastreia a origem para loggar erros.

```rust
enum TokenClass {
    Number,
    LeftParentheses,
    RightParentheses,
    SumOperator,
    SubOperator,
    DivOperator,
    MulOperator,
    Space,
    NewLine,
}
```

### expression

A AST é representada pelo enum `Expression`:

- `NumberLiteral(i32)` — folha com um número inteiro.
- `BinOperation { left_value, operator, right_value }` — operação binária com filhos em `Box<Expression>`.

```rust
enum Expression {
    NumberLiteral(i32),
    BinOperation {
        left_value: Box<Expression>,
        operator: Operator,
        right_value: Box<Expression>,
    },
}
```

Os operadores são modelados pelo enum `Operator` (`Sum`, `Sub`, `Div`, `Mul`).

A gramática reconhecida segue o padrão `(expressão operador expressão)`, com parênteses aninhados. Espaços são ignorados durante a análise.


Observações 
-----------

- A gramatica é limitada, não suporta expressões sem parenteses como `(3 + 4) * 5` ou `3 + 4 * 5`.


Geração
-------

Apartir da arvore expressão o conteudo assembly é incrementado recursivamente para cada nó da árvore no padrão: 

1. se for um número literal, push o valor para a pilha

2. se for uma operação binária, pop os dois valores da pilha, execute a operação e push o resultado para a pilha

Como executar
-------------

Requisito: [Rust](https://www.rust-lang.org/) instalado.

Executar o compilador sobre um arquivo de entrada:

```sh
cargo run <CAMINHO_DO_ARQUIVO>
```

## testar output assembly: 

```sh
as -o output/target_code.o output/target_code.s -Itemplates && ld -o output/target_code output/target_code.o && output/target_code
```


Testes de sucesso
-----------------

```sh
cargo run test/success/source_code_1
cargo run test/success/source_code_2
cargo run test/success/source_code_3
```

Testes de erro
--------------

```sh
cargo run test/error/source_code_1
cargo run test/error/source_code_2
cargo run test/error/source_code_3
```

