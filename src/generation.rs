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
    match name.as_str() {
        "push" => {
            if parameters.len() != 2 {
                panic!("push expects 2 arguments");
            }
            evaluate_expression(&parameters[1], code);
            code.push_str("push %rax\n");
            evaluate_expression(&parameters[0], code);
            code.push_str("mov %rax, %rdi\n");
            code.push_str("pop %rsi\n");
            code.push_str("call array_push\n");
        }
        "len" => {
            if parameters.len() != 1 {
                panic!("len expects 1 argument");
            }
            evaluate_expression(&parameters[0], code);
            code.push_str("mov (%rax), %rax\n");
            code.push_str("mov (%rax), %rax\n");
        }
        "pop" => {
            if parameters.len() != 1 {
                panic!("pop expects 1 argument");
            }
            evaluate_expression(&parameters[0], code);
            code.push_str("mov %rax, %rdi\n");
            code.push_str("call array_pop\n");
        }
        _ => {
            for expression in parameters.iter().rev() {
                evaluate_expression(expression, code);
                code.push_str("push %rax\n");
            }
            code.push_str(&format!("call {}\n", name));
            code.push_str(&format!("add ${}, %rsp\n", 8 * parameters.len()));
        }
    }
}

fn load_handle_to_rax(array: &String, code: &mut Code) {
    let location = find_variable_stack_index(array, &code.local, &code.global);
    code.push_str(&format!("mov {}, %rax\n", location));
}

fn load_data_ptr_from_handle_in_rax(code: &mut Code) {
    code.push_str("mov (%rax), %rax\n");
}

fn evaluate_expression(expression: &Expression, code: &mut Code) {
    match expression {
        Expression::NumberLiteral(number) => {
            code.push_str(&format!("mov ${}, %rax\n", number));
        }
        Expression::TextLiteral(text) => {
            let chars: Vec<i32> = text.chars().map(|c| c as i32).collect();
            let n = chars.len();
            code.push_str(&format!("mov ${}, %rdi\n", n));
            code.push_str("call array_new\n");
            code.push_str("push %rax\n");
            for (i, ch) in chars.iter().enumerate() {
                code.push_str(&format!("mov ${}, %rdx\n", ch));
                code.push_str("mov (%rsp), %rax\n");
                code.push_str("mov (%rax), %rax\n");
                code.push_str(&format!("mov %rdx, {}(%rax)\n", 16 + i * 8));
            }
            code.push_str("pop %rax\n");
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
                Operator::Sub => {
                    code.push_str("neg %rax\n");
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
            code.push_str("pop %rbx\n");
            match operator {
                Operator::Sum => {
                    code.push_str("add %rbx, %rax\n");
                }
                Operator::Sub => {
                    code.push_str("sub %rax, %rbx\n");
                    code.push_str("mov %rbx, %rax\n");
                }
                Operator::Mul => {
                    code.push_str("imul %rbx, %rax\n");
                }
                Operator::Div | Operator::Mod => {
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
                Operator::And | Operator::Or => {
                    code.push_str("test %rbx, %rbx\n");
                    code.push_str("setnz %bl\n");
                    code.push_str("movzbq %bl, %rbx\n");
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
        Expression::Index { array, index } => {
            evaluate_expression(index, code);
            code.push_str("push %rax\n");
            load_handle_to_rax(array, code);
            load_data_ptr_from_handle_in_rax(code);
            code.push_str("pop %rcx\n");
            code.push_str("imul $8, %rcx\n");
            code.push_str("add $16, %rcx\n");
            code.push_str("add %rcx, %rax\n");
            code.push_str("mov (%rax), %rax\n");
        }
        Expression::ArrayLiteral(elements) => {
            let n = elements.len();
            code.push_str(&format!("mov ${}, %rdi\n", n));
            code.push_str("call array_new\n");
            code.push_str("push %rax\n");
            for (i, element) in elements.iter().enumerate() {
                evaluate_expression(element, code);
                code.push_str("mov %rax, %rdx\n");
                code.push_str("mov (%rsp), %rax\n");
                code.push_str("mov (%rax), %rax\n");
                code.push_str(&format!("mov %rdx, {}(%rax)\n", 16 + i * 8));
            }
            code.push_str("pop %rax\n");
        }
    }
}

fn store_rax_to_variable(name: &String, code: &mut Code) {
    let destination = find_variable_stack_index(name, &code.local, &code.global);
    code.push_str(&format!("mov %rax, {}\n", destination));
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
        Command::Attribution { target, expression } => match target {
            AssignTarget::Variable(variable) => {
                evaluate_expression(expression, code);
                store_rax_to_variable(variable.get_lexema(), code);
            }
            AssignTarget::Index { array, index } => {
                evaluate_expression(expression, code);
                code.push_str("push %rax\n");
                evaluate_expression(index, code);
                code.push_str("push %rax\n");
                let variable_name = array.get_lexema();
                load_handle_to_rax(variable_name, code);
                load_data_ptr_from_handle_in_rax(code);
                code.push_str("pop %rcx\n");
                code.push_str("imul $8, %rcx\n");
                code.push_str("add $16, %rcx\n");
                code.push_str("add %rcx, %rax\n");
                code.push_str("pop %rdx\n");
                code.push_str("mov %rdx, (%rax)\n");
            }
        },
        Command::Return { expression } => {
            evaluate_expression(expression, code);
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
            Identifier::Variable(Variable { name, expression, .. }) => {
                evaluate_expression(expression, code);
                store_rax_to_variable(name.get_lexema(), code);
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

fn count_local_variables(function: &Function) -> usize {
    function
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
        .count()
}

fn find_variable_stack_index(name: &String, function: &Function, global: &Vec<String>) -> String {
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

    if let Some(mut index) = function
        .parameters
        .iter()
        .position(|p| p.name.get_lexema() == name)
    {
        index = 8 * count_local_variables(function) + 8 * 2 + 8 * index;
        return format!("{}(%rbp)", index);
    }

    if global.iter().any(|g| g == name) {
        return name.to_string();
    }

    panic!("cannot use a variable without declare it first")
}

fn generate_functions(program: &Program) -> String {
    let mut text: String = String::new();
    let mut global_variables_names: Vec<String> = Vec::new();
    for identifier in &program.declarations {
        if let Identifier::Variable(variable) = identifier {
            let name = variable.name.get_lexema().to_string();
            global_variables_names.push(name);
        }
    }
    for identifier in &program.declarations {
        if let Identifier::Function(function) = identifier {
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

    let global_scope = Function {
        name: Token {
            class: crate::lexical::TokenClass::KeyWord,
            column: 0,
            line: 0,
            lexema: String::from(""),
        },
        parameters: vec![],
        return_type: Type::Num,
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
            store_rax_to_variable(variable.name.get_lexema(), &mut code);
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
