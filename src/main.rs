use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Not;

fn main() {
    zeller()
}

fn zeller() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Uso: {} DD MM CC YY", args[0]);
        return;
    }
    let dd: i32 = args[1].parse().expect("DD inválido");
    let mm: i32 = args[2].parse().expect("MM inválido");
    let cc: i32 = args[3].parse().expect("CC inválido");
    let yy: i32 = args[4].parse().expect("YY inválido");

    let week_day = (dd + ((13 * (mm + 1)) / 5) + yy + (yy / 4) + (cc / 4) - 2 * cc) % 7;

    println!("{}", week_day);
}

fn compile_soruce_code() {
    let arg: String = env::args()
        .nth(1)
        .expect("Please provide the source code file path as argument.");
    let source_code: String = fs::read_to_string(arg).expect("Error reading file");
    // validate_code(&source_code);
    let number: i32 = extract_number(&source_code);

    let target_code: String = ASSEMBLY.replace("{NUMBER}", &number.to_string());
    const OUTPUT_PATH: &str = "target_code";
    let mut file = File::create(OUTPUT_PATH).expect("Error creating file");
    file.write_all(target_code.as_bytes())
        .expect("Error writing file");
}

// fn validate_code(code: &str) {}

fn extract_number(code: &str) -> i32 {
    let mut number_str = String::new();
    for c in code.chars() {
        if c.is_digit(10) {
            number_str.push(c);
        }
    }
    if number_str.is_empty().not() {
        return number_str.parse::<i32>().unwrap();
    }
    panic!("No valid number found.");
}

const ASSEMBLY: &str = r#"
#
# modelo de saida para o compilador
#
.section .text
.globl _start
_start:
mov ${NUMBER}, %rax 
call imprime_num
call sair
.include "assembly/runtime.s"
"#;
