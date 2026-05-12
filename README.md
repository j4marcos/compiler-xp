compilador em rust com cargo
aluno: 20230013339 - João Marcos Cunha Santos

-> traduzir texto de numero inteiro para codigo objeto assembly

- test success:
cargo run source_code
as --64 -o code_exe.o target_code
ld -o code_exe code_exe.o
./code_exe

- test error:
cargo run source_code-error
