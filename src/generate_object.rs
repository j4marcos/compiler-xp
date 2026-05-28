


fn generate_code_from_number(number: i32) {
    let target_code: String = ASSEMBLY.replace("{NUMBER}", &number.to_string());
    const OUTPUT_PATH: &str = "target_code";
    let mut file = File::create(OUTPUT_PATH).expect("Error creating file");
    file.write_all(target_code.as_bytes())
        .expect("Error writing file");
}

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
