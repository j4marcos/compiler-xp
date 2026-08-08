use crate::{lexical::Token, syntax::*};

const OUTPUT_TEMPLATE: &str = r#".section .bss
{bss}
.section .text
.globl _start
{lib}
_start:
{main}
call imprime_num
call sair
.include "runtime.s"
"#;

struct Code {
    text: String,
    local: Function,
    global: Vec<String>,
}

impl Code {
    fn push_str(&mut self, string: &str) {
        self.text.push_str(string)
    }
    fn len(&self) -> usize {
        self.text.len()
    }
}

fn generate_function_call(name: &String, parameters: &Vec<Expression>, code: &mut Code) {
    for expression in parameters.iter().rev() {
        evaluate_expression(expression, code);
        code.push_str("push %rax\n");
    }
    code.push_str(&format!("call {}\n", name));
    code.push_str(&format!("add ${}, %rsp\n", 8 * parameters.len()));
}

/// Evaluates an expression leaving the result in %rax.
fn evaluate_expression(expression: &Expression, code: &mut Code) {
    match expression {
        Expression::NumberLiteral(number) => {
            code.push_str(&format!("mov ${}, %rax\n", number));
        }
        Expression::Identifier(name) => {
            let rbp_position = find_variable_stack_index(name, &code.local, &code.global);
            code.push_str(&format!("mov {}, %rax\n", rbp_position));
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
        Expression::FunctionCall { name, parameters } => {
            generate_function_call(name, parameters, code)
        }
    }
}

fn evaluate_command(command: &Command, code: &mut Code) {
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
            let destination =
                find_variable_stack_index(variable.get_lexema(), &code.local, &code.global);
            code.push_str(&format!("mov %rax, {}\n", destination));
        }
        Command::Return { expression } => {
            evaluate_expression(expression, code);

            // clear_local_stack
            code.push_str(&format!(
                "add ${}, %rsp\n",
                8 * count_local_variables(&code.local)
            ));

            code.push_str("pop %rbp\n");
            code.push_str("ret\n");
        }
        Command::Print { expression } => {
            evaluate_expression(expression, code);
            code.push_str("call imprime_num\n");
        }
        Command::FunctionCall { name, parameters } => {
            generate_function_call(name, parameters, code);
        }
        Command::Declaration { identifier } => match identifier {
            Identifier::Variable(Variable { name, expression }) => {
                evaluate_expression(expression, code);
                let rbp_position =
                    find_variable_stack_index(name.get_lexema(), &code.local, &code.global);
                code.push_str(&format!("mov %rax, {}\n", rbp_position));
            }
            Identifier::Function(_) => {
                panic!("function cant be declared inside another function");
            }
        },
    }
}

fn generate_commands(commands: &Vec<Command>, code: &mut Code) {
    for cmd in commands {
        evaluate_command(cmd, code);
    }
}

fn generate_global_variables(program: &Program) -> String {
    let mut text = String::new();
    for identifier in &program.declarations {
        if let Identifier::Variable(variable) = identifier {
            let name = variable.name.get_lexema();
            text.push_str(&format!(".lcomm {}, 8\n", name));
        }
    }
    text
}

// é feito varias vezes na recurção o calculo, seria bom salvar o valor na struct Function
fn count_local_variables(function: &Function) -> usize {
    return function
        .code_block
        .commands
        .iter()
        .filter(|cmd| {
            matches!(
                cmd,
                Command::Declaration {
                    identifier: Identifier::Variable(_),
                }
            )
        })
        .count();
}

// ŕ feito varias vezes na recursão o calculo, seria bom salvar o valor dentro da struct da Variable;
fn find_variable_stack_index(name: &String, function: &Function, global: &Vec<String>) -> String {
    // procurar em qual lista de declarações esta a variavel

    // local
    if let Some(index) = function
        .code_block
        .commands
        .iter()
        .filter_map(|cmd| match cmd {
            Command::Declaration {
                identifier: Identifier::Variable(v),
            } => Some(v.name.get_lexema()),
            _ => None,
        })
        .position(|n| n == name)
    {
        return format!("{}(%rbp)", 8 * index);
    }

    // params
    if let Some(mut index) = function
        .parameters
        .iter()
        .position(|p| p.get_lexema() == name)
    {
        index = 8 * count_local_variables(function) + 8 * 2 + 8 * index;

        return format!("{}(%rbp)", index);
    }

    // global
    if global.iter().find(|g| g == &name).is_some() {
        return name.to_string();
    }

    panic!("cannot use a variable without declare it first")

    // para usar uma variavel -> pega o valor da expressão e coloca em rax, de rax identifica qual é a posição na pilha dessa variavel:
    // variavel parametro da função : posição em RBP + numero de variaveis locais * 8 + 16 (metadata) + 8 * index da parametro
    // variavel local declarada : posição em RBP + ordem de declaração da variavel no local * 8
}

fn generate_functions(program: &Program) -> String {
    let mut text: String = String::new();
    let mut global_variables_names: Vec<String> = Vec::new();
    for identifier in &program.declarations {
        match identifier {
            Identifier::Variable(variable) => {
                let name = variable.name.get_lexema().to_string();
                global_variables_names.push(name);
            }
            Identifier::Function(function) => {
                let mut code = Code {
                    local: function.clone(),
                    global: global_variables_names.clone(),
                    text: String::new(),
                };

                let name = code.local.name.get_lexema();
                text.push_str(&format!("{}:\n", name));
                text.push_str("push %rbp\n");
                text.push_str(&format!(
                    "sub ${}, %rsp\n",
                    8 * count_local_variables(function)
                ));
                text.push_str("mov %rsp, %rbp\n");
                generate_commands(&function.code_block.commands, &mut code);
                text.push_str(&code.text);
            }
        }
    }
    text
}

fn generate_global_variables_inicialization(Program { declarations }: &Program, text: &mut String) {
    let global_names: Vec<String> = declarations
        .iter()
        .filter_map(|d| match d {
            Identifier::Variable(v) => Some(v.name.get_lexema().to_string()),
            _ => None,
        })
        .collect();

    // inicializa variáveis globais
    let global_scope = Function {
        name: Token {
            class: crate::lexical::TokenClass::KeyWord,
            column: 0,
            line: 0,
            lexema: String::from(""),
        },
        parameters: vec![],
        code_block: CodeBlock { commands: vec![] },
    };
    for identifier in declarations {
        if let Identifier::Variable(variable) = identifier {
            let mut code = Code {
                local: global_scope.clone(),
                global: global_names.clone(),
                text: String::new(),
            };
            evaluate_expression(&variable.expression, &mut code);
            code.push_str(&format!("mov %rax, {}\n", variable.name.get_lexema()));
            text.push_str(&code.text);
        }
    }
}

fn generate_call_main(program: &Program) -> String {
    let mut text = String::new();
    generate_global_variables_inicialization(program, &mut text);

    let Some(Identifier::Function(main)) = program
        .declarations
        .iter()
        .find(|d| matches!(d, Identifier::Function(f) if f.name.get_lexema() == "main"))
    else {
        panic!("program must have a main function")
    };

    text.push_str("call main\n");
    if !main.parameters.is_empty() {
        text.push_str(&format!("add ${}, %rsp\n", 8 * main.parameters.len()));
    }

    text
}

pub fn generate_assembly(program: &Program) -> String {
    OUTPUT_TEMPLATE
        .replace("{bss}", &generate_global_variables(program))
        .replace("{lib}", &generate_functions(program))
        .replace("{main}", &generate_call_main(program))
}
