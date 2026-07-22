use crate::syntax::{CodeBlock, Command, Expression, Program};
use std::collections::HashSet;

fn handle_semantic_error(reason: &str) -> ! {
    eprintln!("Error semantic: {}", reason);
    std::process::exit(1);
}

fn check_expression(expression: &Expression, declared: &HashSet<String>) {
    match expression {
        Expression::NumberLiteral(_) => {}
        Expression::Identifier(name) => {
            if !declared.contains(name) {
                handle_semantic_error(&format!("variable '{}' used before declaration", name));
            }
        }
        Expression::UnaryOperation { operand, .. } => {
            check_expression(operand, declared);
        }
        Expression::BinOperation {
            left_value,
            right_value,
            ..
        } => {
            check_expression(left_value, declared);
            check_expression(right_value, declared);
        }
    }
}

fn check_command(command: &Command, declared: &HashSet<String>) {
    match command {
        Command::Attribution {
            variable,
            expression,
        } => {
            let lexema = variable.get_lexema();
            if !declared.contains(lexema) {
                handle_semantic_error(&format!("variable '{}' used before declaration", lexema));
            }
            check_expression(&expression, declared);
        }
        Command::If {
            condition,
            true_block,
            false_block,
        } => {
            check_expression(&condition, declared);
            check_code_block(true_block, declared);
            check_code_block(false_block, declared);
        }
        Command::While { condition, block } => {
            check_expression(&condition, declared);
            check_code_block(block, declared)
        }
    }
}

fn check_code_block(block: &CodeBlock, declared: &HashSet<String>) {
    for command in &block.0 {
        check_command(command, declared);
    }
    if let Some(expression) = &block.1 {
        check_expression(&expression, declared);
    }
}

/// - Validates that no variable is used before it has been declared.
pub fn validate_program(program: &Program) {
    let mut declared: HashSet<String> = HashSet::new();

    for variable in &program.declarations {
        let name = variable.identifier.get_lexema().to_string();

        if declared.contains(&name) {
            handle_semantic_error(&format!("variable '{}' already declared", name));
        }

        check_expression(&variable.expression, &declared);
        declared.insert(name);
    }

    for command in &program.commands {
        check_command(command, &declared);
    }
    check_expression(&program.expression, &declared)
}
