compilador em rust com cargo
aluno: 20230013339 - João Marcos Cunha Santos


-> achar a Congruência de Zeller

- test rust 
cargo run -- 25 05 20 26

- test asselmbly
cd /assembly
as --64 zeller.s -o zeller.o 
ld zeller.o -o zeller
./zeller 


-> traduzir texto de numero inteiro para codigo objeto assembly

- test success:
cargo run source_code
as --64 -o code_exe.o target_code
ld -o code_exe code_exe.o
./code_exe

- test error:
cargo run source_code-error

