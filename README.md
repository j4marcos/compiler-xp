
Compilador em Rust
===================

Grupo: 20230013339 — João Marcos Cunha Santos

Analisador léxico de expressões
-------------------------------

O analisador identifica e classifica a lista de tokens presente em um arquivo: números, operadores, pontuação e espaços. 

- O módulo de tokenização é `src/tokenize.rs`.

testar
----------

Executar o analisador léxico sobre um arquivo de entrada (cargo run <NOME_ARQUIVO>):

```sh
cargo run source_code
```

saída (sucesso):

```
Token::LeftParentheses (() at 1:1
Token::Number (123) at 1:2
Token::Space ( ) at 1:5
Token::SumOperator (+) at 1:6
Token::Space ( ) at 1:7
Token::Number (098) at 1:8
Token::RightParentheses ()) at 1:11
Token::Space ( ) at 1:12
Token::SubOperator (-) at 1:13
Token::Space ( ) at 1:14
Token::Number (33) at 1:15
Token::Space ( ) at 1:17
Token::DivOperator (/) at 1:18
Token::Space (    ) at 1:19
Token::LeftParentheses (() at 1:23
Token::Number (3) at 1:24
Token::Space ( ) at 1:25
Token::SumOperator (+) at 1:26
Token::Space ( ) at 1:27
Token::Number (50) at 1:28
Token::RightParentheses ()) at 1:30
```

Erros de análise
-----------------

Se o arquivo de entrada contiver caracteres inválidos, o analisador emite uma mensagem de erro, por exemplo:

```sh
cargo run source_code-error
```

saída (erro):

```
Error: invalid character 'x' at 1:6
```
Outro exemplo
```sh
cargo run source_code-error-2
```

saída (erro):


```
Error: invalid character 'a' at 1:4
```
