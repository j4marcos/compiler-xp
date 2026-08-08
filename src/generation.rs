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
fn evaluate_expression(expression: &Expression, code: &mut String) {
    match expression {
        Expression::NumberLiteral(number) => {
            code.push_str(&format!("mov ${}, %rax\n", number));
        }
        Expression::Identifier(name) => {
            code.push_str(&format!("mov {}, %rax\n", name));
        }
        Expression::UnaryOperation { operator, operand } => {
            evaluate_expression(operand, code);
            match operator {
                Operator::Not => {
                    code.push_str("cmp $0, %rax\n");
                    code.push_str("sete %al\n");
                    code.push_str("movzbq %al, %rax\n");
                }
                _ => unreachable!("invalid unary operator"),
            }
        }
        Expression::BinOperation {
            left_value,
            operator,
            right_value,
        } => {
            evaluate_expression(left_value, code);
            code.push_str("push %rax\n");
            evaluate_expression(right_value, code);
            // left in %rbx, right in %rax
            code.push_str("pop %rbx\n");
            match operator {
                Operator::Sum => {
                    code.push_str("add %rbx, %rax\n");
                }
                Operator::Sub => {
                    // rax = left - right
                    code.push_str("sub %rax, %rbx\n");
                    code.push_str("mov %rbx, %rax\n");
                }
                Operator::Mul => {
                    code.push_str("imul %rbx, %rax\n");
                }
                Operator::Div | Operator::Mod => {
                    // rax = left / right ; remainder in %rdx
                    code.push_str("mov %rax, %rcx\n");
                    code.push_str("mov %rbx, %rax\n");
                    code.push_str("cqo\n");
                    code.push_str("idiv %rcx\n");
                    if matches!(operator, Operator::Mod) {
                        code.push_str("mov %rdx, %rax\n");
                    }
                }
                Operator::Equal
                | Operator::LessThan
                | Operator::LessEqual
                | Operator::GreaterThan
                | Operator::GreaterEqual => {
                    code.push_str("xor %rcx, %rcx\n");
                    code.push_str("cmp %rax, %rbx\n");

                    match operator {
                        Operator::Equal => code.push_str("sete %cl\n"),
                        Operator::LessThan => code.push_str("setl %cl\n"),
                        Operator::LessEqual => code.push_str("setle %cl\n"),
                        Operator::GreaterThan => code.push_str("setg %cl\n"),
                        Operator::GreaterEqual => code.push_str("setge %cl\n"),
                        _ => unreachable!(),
                    }
                    code.push_str("mov %rcx, %rax\n");
                }
                // any true - 0 false
                Operator::And | Operator::Or => {
                    // rbx to boolean
                    code.push_str("test %rbx, %rbx\n");
                    code.push_str("setnz %bl\n");
                    code.push_str("movzbq %bl, %rbx\n");
                    // rax to boolean
                    code.push_str("test %rax, %rax\n");
                    code.push_str("setnz %al\n");
                    code.push_str("movzbq %al, %rax\n");

                    match operator {
                        Operator::And => code.push_str("and %rbx, %rax\n"),
                        Operator::Or => code.push_str("or %rbx, %rax\n"),
                        _ => unreachable!(),
                    }
                }
                Operator::Not => unreachable!("invalid binary operator"),
            }
        }
        Expression::FunctionCall { name, parameters } => todo!(),
    }
}

fn evaluate_command(command: &Command, code: &mut String) {
    match command {
        Command::If {
            condition,
            true_block,
            false_block,
        } => {
            let label = code.len();
            evaluate_expression(condition, code);
            code.push_str("cmp $0, %rax\n");
            code.push_str(&format!("jz Lfalso{}\n", label));
            generate_commands(&true_block.commands, code);
            code.push_str(&format!("jmp Lfim{}\n", label));
            code.push_str(&format!("Lfalso{}:\n", label));
            generate_commands(&false_block.commands, code);
            code.push_str(&format!("Lfim{}:\n", label));
        }
        Command::While { condition, block } => {
            let label = code.len();
            code.push_str(&format!("Linicio{}:\n", label));
            evaluate_expression(condition, code);
            code.push_str("cmp $0, %rax\n");
            code.push_str(&format!("jz Lfim{}\n", label));
            generate_commands(&block.commands, code);
            code.push_str(&format!("jmp Linicio{}\n", label));
            code.push_str(&format!("Lfim{}:\n", label));
        }
        Command::Attribution {
            variable,
            expression,
        } => {
            evaluate_expression(expression, code);
            code.push_str(&format!("mov %rax, {}\n", variable.get_lexema()));
        }
        Command::Return { expression } => {
            evaluate_expression(expression, code);
            // pula pro final da func
            code.push_str("jmp Lretorno\n");
        }
        Command::Print { expression } => {
            evaluate_expression(expression, code);
            code.push_str("call imprime_num\n");
        }
        Command::FunctionCall { name, parameters } => todo!(),
        Command::Declaration { identifier } => todo!(),
    }
}

fn generate_commands(commands: &Vec<Command>, code: &mut String) {
    for cmd in commands {
        evaluate_command(cmd, code);
    }
}

fn generate_bss(program: &Program) -> String {
    let mut bss = String::new();
    for identifier in &program.declarations {
        if let Identifier::Variable(variable) = identifier {
            let name = variable.token.get_lexema();
            bss.push_str(&format!(".lcomm {}, 8\n", name));
        }
    }
    bss
}

fn generate_lib(program: &Program) -> String {
    let mut lib = String::new();
    for identifier in &program.declarations {
        if let Identifier::Function(Function {
            token,
            parameters,
            code_block,
        }) = identifier
        {
            let name = token.get_lexema();


            lib.push_str(&format!("{}:\n", name));
            lib.push_str("push %rbp\n");
            lib.push_str("mov %rsp, %rbp\n");
        }
    }
    lib
}

fn generate_main(program: &Program) -> String {
    let mut code = String::new();

    generate_declarations(&program.declarations, &mut code);
    generate_commands(&program.commands, &mut code);
    // fim da main
    code.push_str("Lretorno:\n");
    code
}

fn generate_declarations(declarations: &Vec<Identifier>, code: &mut String) {
    for identifier in declarations {
        match identifier {
            Identifier::Function(Function {
                token,
                parameters,
                code_block,
            }) => {
                // nessa versão, não pode declarar funções no corpo da main
            }
            Identifier::Variable(Variable { token, expression }) => {
                let name = token.get_lexema();
                evaluate_expression(&expression, code);
                code.push_str(&format!("mov %rax, {}\n", name));
            }
        }
    }
}

pub fn generate_assembly(program: &Program) -> String {
    OUTPUT_TEMPLATE
        .replace("{bss}", &generate_bss(program))
        .replace("{lib}", &generate_lib(program))
        .replace("{main}", &generate_main(program))
}
