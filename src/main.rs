use std::env;
use std::fs;
mod generation;
mod lexical;
mod semantic;
mod syntax;

fn main() {
    compile_source_code()
}

fn compile_source_code() {
    let arg: String = env::args()
        .nth(1)
        .expect("Please provide the source code file path as argument.");

    let source_code: String = fs::read_to_string(arg).expect("Error reading file");
    let tokens = lexical::extract_tokens(&source_code);
    let program = syntax::build_program(tokens);
    semantic::validate_program(&program);
    println!("{:?}",program);
    let assembly = generation::generate_assembly(&program);
    print!("{}", assembly);
    let output_path = "output/target_code.s";
    fs::create_dir_all("output").expect("Error creating output directory");
    fs::write(output_path, assembly).expect("Error wrinting assembly file");
}
