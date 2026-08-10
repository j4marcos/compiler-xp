Compiler-xp - experimental compiler in rust
===================

Grupo: 20230013339 — João Marcos Cunha Santos  
Repositorio: https://github.com/j4marcos/compiler-xp.git

Visão geral
-----------

Compilador lê um programa da linguagem **Func** a partir de um arquivo de entrada e gera código assembly x86-64 (sintaxe AT&T), com o fluxo:

1. **Análise léxica** (`src/lexical.rs`) — classifica tokens (números, operadores, identificadores, palavras-chave, blocos, etc.).
2. **Análise sintática** (`src/syntax.rs`) — monta a AST de expressões, declarações e funções.
3. **Resolução de imports** (`src/resolve.rs`) — carrega bibliotecas e mescla símbolos `alias::nome` no programa.
4. **Análise semântica** (`src/semantic.rs`) — valida tipos, declarações, escopos e existência de `main` com `return` final.
5. **Geração de código** (`src/generation.rs`) — emite assembly; `_start` inicializa globais, chama `main`, imprime o valor em `%rax` via runtime e encerra.

O assembly é impresso no terminal e salvo em `output/target_code.s`.


### Linguagem Func

Tipos: `num`, `list`, `bool`, `text`. Declarações tipadas, parâmetros tipados, métodos (`x.f(args)` → `f(x, args)`), arrays dinâmicos com handle, `print`, comentários `//`, **import** de bibliotecas. O programa deve ter `func main()` e o último comando de `main` deve retornar `num`/`bool`.

```
<programa> ::= <import>* <decl>*
<import>     ::= 'import' <ident> 'from' <string>
<decl>       ::= <vardecl> | <fundecl>
<vardecl>    ::= <tipo> <ident> '=' <exp> ';'
<tipo>       ::= 'num' | 'list' | 'bool' | 'text'
<fundecl>    ::= 'func' <ident> '(' <arglist>? ')' <tipo> '{' <cmd>* '}'
<arglist>    ::= <arg> | <arg> ',' <arglist>
<arg>        ::= <tipo> <ident>
<cmd>        ::= <if> | <while> | <atrib> | <return> | <print> | <callstmt> | <vardecl>
<print>      ::= 'print' <exp> ';'
<callstmt>   ::= <exp> ';'?          # chamada/método
<if>         ::= 'if' <exp> '{' <cmd>* '}' 'else' '{' <cmd>* '}'
<while>      ::= 'while' <exp> '{' <cmd>* '}'
<atrib>      ::= <ident> '=' <exp> ';'
             | <ident> '[' <exp> ']' '=' <exp> ';'
             | <ident> '+=' <exp> ';' | ... | <ident> '++' ';'
<return>     ::= 'return' <exp> ';'

<ident>    ::= letter (letter|digit)* ('::' letter (letter|digit)*)*
<exp>      ::= <exp_or> ('.' <ident> '(' <params>? ')')*
...
<prim>     ::= <num> | <ident> | <ident> '[' <exp> ']' | '(' <exp> ')' | <funcall>
             | '[' <params>? ']' | <exp> '.' 'len' | <exp> '.' 'len' '(' ')'
             | 'true' | 'false' | <string>
```

**Tipos**
- `num` — inteiro 64-bit
- `bool` — mesmo valor numérico; em `if`/`while`, `0` é falso e qualquer outro é verdadeiro
- `list` — array dinâmico (handle → `[len][cap][data]`); cria com `[]` ou `[1, 2, 3]`
- `text` — mesmo layout de `list`; literais `"abc"` viram códigos ASCII

**Métodos:** `recv.nome(args)` vira `nome(recv, args)`. Encadeável. O builtin `push(list|text, num)` devolve o handle. Comprimento via **`a.len`** ou `a.len()` (não existe mais a forma `len(a)` como keyword).

**Imports:** `import ss from "./sum_lists"` carrega o arquivo (path relativo ao fonte). A lib pode ter funções e globais, **sem** `main` e **sem** imports aninhados. Símbolos entram no escopo como `ss::count` / `ss::var`. Chamadas: `ss::count(x)` ou `x.ss::count()`.

Bibliotecas prontas em `lib/`:
- `numbers.lib` — `abs`, `max`, `min`, `sign`, `clamp`, `pow`, `isEven`, `isOdd`, `xor`, `factorial`
- `lists.lib` — `count`, `sum`, `product`, `maximum`, `minimum`, `last`, `contains`, `indexOf`, `reverse`, `scale`, `pushAll`, `range`, `isEmpty` (também serve para `text`)

Exemplo:

```
import ss from "./sum_lists"

func main() num {
  list x = [1, 2, 3];
  return x.ss::count();
}
```

```
func abs(num x) num {
  if x < 0 {
    return 0 - x;
  } else {
    return x;
  }
}

func main() num {
  num x = -3;
  list a = [1, 2];
  a.push(3);
  text t = "hi";
  bool ok = true;
  return x.abs() + a.len + t[0];
}
```


Estrutura
---------

### Token

Keywords: `if`, `else`, `while`, `return`, `print`, `and`, `or`, `not`, `func`, `main`, `num`, `list`, `bool`, `text`, `true`, `false`, `import`, `from`.

### Expression / Command

```rust
enum Type { Num, List, Bool, Text }

enum Expression {
    NumberLiteral(i32),
    Identifier(String),
    UnaryOperation { operator, operand },
    BinOperation { left_value, operator, right_value },
    FunctionCall { name, parameters }, // inclui métodos desaçucarados
    Index { array, index },
    ArrayLiteral(Vec<Expression>),
    TextLiteral(String),
}

enum Command {
    If { condition, true_block, false_block },
    While { condition, block },
    Attribution { target, expression },
    FunctionCall { name, parameters },
    Return { expression },
    Print { expression },
    Declaration { identifier },
}
```


Geração
-------

Expressões deixam o resultado em `%rax`.

1. literal / variável → `mov` para `%rax`
2. binária / unária (`not`, `-`)
3. `if` / `while` → `cmp` + saltos
4. chamada → empilha args (exceto builtins `push` / `len` via `.len`)
5. arrays → handle + `array_new` / `array_push` no runtime (`brk`)
6. `_start` → globais, `call main`, `imprime_num`, `sair`


Como executar
-------------

```sh
cargo run <CAMINHO_DO_ARQUIVO>
as -o output/target_code.o output/target_code.s -Itemplates && ld -o output/target_code output/target_code.o && output/target_code
```


Testes de sucesso
-----------------

```sh
for i in $(seq 1 30); do
  cargo run test/success/source_code_$i
  as -o output/target_code.o output/target_code.s -Itemplates
  ld -o output/target_code output/target_code.o
  output/target_code
done
```

| Arquivo | Cobre | Resultado |
|---------|--------|-----------|
| `24`–`27` | list / len / push / alias / param | `3`, `14`, `2`, `5` |
| `28` | método `x.abs()` | `3` |
| `29` | chain `mult.push.sum` | `14` |
| `30` | bool / text / `[]` | `246` |
| `34` | `import` + `x.ss::count()` | `3` |
| `35`–`36` | `lib/numbers.lib` | `42`, `12` |
| `37`–`39` | `lib/lists.lib` (list/text) | `42`, `15`, `111` |


Testes de erro
--------------

```sh
for i in $(seq 1 10); do cargo run test/error/source_code_$i; done
```
