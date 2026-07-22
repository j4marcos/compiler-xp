use crate::syntax::*;

const OUTPUT_TEMPLATE: &str = r#".section .bss
{bss}

.section .text
.globl _start
_start:
{code}
call imprime_num
call sair
.include "runtime.s"
"#;

/// Evaluates an expression leaving the result in %rax.
fn evaluate_expression(expression: &Expression, body: &mut String) {
    match expression {
        Expression::NumberLiteral(number) => {
            body.push_str(&format!("mov ${}, %rax\n", number));
        }
        Expression::Identifier(name) => {
            body.push_str(&format!("mov {}, %rax\n", name));
        }
        Expression::UnaryOperation { operator, operand } => {
            evaluate_expression(operand, body);
            match operator {
                Operator::Not => {
                    body.push_str("cmp $0, %rax\n");
                    body.push_str("sete %al\n");
                    body.push_str("movzbq %al, %rax\n");
                }
                _ => unreachable!("invalid unary operator"),
            }
        }
        Expression::BinOperation {
            left_value,
            operator,
            right_value,
        } => {
            evaluate_expression(left_value, body);
            body.push_str("push %rax\n");
            evaluate_expression(right_value, body);
            // left in %rbx, right in %rax
            body.push_str("pop %rbx\n");
            match operator {
                Operator::Sum => {
                    body.push_str("add %rbx, %rax\n");
                }
                Operator::Sub => {
                    // rax = left - right
                    body.push_str("sub %rax, %rbx\n");
                    body.push_str("mov %rbx, %rax\n");
                }
                Operator::Mul => {
                    body.push_str("imul %rbx, %rax\n");
                }
                Operator::Div => {
                    // rax = left / right
                    body.push_str("mov %rax, %rcx\n");
                    body.push_str("mov %rbx, %rax\n");
                    body.push_str("cqo\n");
                    body.push_str("idiv %rcx\n");
                }
                Operator::Equal | Operator::LessThan | Operator::GreaterThan => {
                    body.push_str("cmp %rax, %rbx\n");
                    match operator {
                        Operator::Equal => body.push_str("sete %al\n"),
                        Operator::LessThan => body.push_str("setl %al\n"),
                        Operator::GreaterThan => body.push_str("setg %al\n"),
                        _ => unreachable!(),
                    }
                    body.push_str("movzbq %al, %rax\n");
                }
                Operator::And => {
                    body.push_str("and %rbx, %rax\n");
                }
                Operator::Or => {
                    body.push_str("or %rbx, %rax\n");
                }
                Operator::Not => unreachable!("invalid binary operator"),
            }
        }
    }
}

fn generate_bss(program: &Program) -> String {
    let mut bss = String::new();
    for variable in &program.declarations {
        let name = variable.identifier.get_lexema();
        bss.push_str(&format!(".lcomm {}, 8\n", name));
    }
    bss
}

fn generate_code(program: &Program) -> String {
    let mut code = String::new();

    for variable in &program.declarations {
        let name = variable.identifier.get_lexema();
        evaluate_expression(&variable.expression, &mut code);
        code.push_str(&format!("mov %rax, {}\n", name));
    }

    evaluate_expression(&program.expression, &mut code);
    code
}

pub fn generate_assembly(program: &Program) -> String {
    OUTPUT_TEMPLATE
        .replace("{bss}", &generate_bss(program))
        .replace("{code}", &generate_code(program))
}
