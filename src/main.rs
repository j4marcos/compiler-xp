use std::env;
use std::fs;
mod zeller;
mod tokenize;

fn main() {
    tokenize_source_code()
}

fn tokenize_source_code() {
    let arg: String = env::args()
        .nth(1)
        .expect("Please provide the source code file path as argument.");

    let source_code: String = fs::read_to_string(arg).expect("Error reading file");
    let tokens = tokenize::extract_tokens(&source_code);
    println!("{}", tokens);
}



