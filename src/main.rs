use std::env;
use std::fs;
// mod zeller;
mod lexical;
mod syntax;
mod generation;

fn main() {
    compile_source_code()
}

fn compile_source_code() {
    let arg: String = env::args()
        .nth(1)
        .expect("Please provide the source code file path as argument.");

    let source_code: String = fs::read_to_string(arg).expect("Error reading file");
    let tokens = lexical::extract_tokens(&source_code);
    let expression = syntax::extract_expression(tokens);
    let assembly = generation::generate_assembly(expression);
    println!("{}", &assembly);
    let output_path = "output/target_code.s";
    // println!("assembly generated at: {}", output_path );
    fs::write(output_path, assembly).expect("Error wrinting assembly");
}



