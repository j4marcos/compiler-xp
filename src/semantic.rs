use crate::syntax::{CodeBlock, Command, Expression, Identifier, Program};
use std::collections::HashSet;

fn handle_semantic_error(reason: &str) -> ! {
    eprintln!("Error semantic: {}", reason);
    std::process::exit(1);
}

fn check_variable_is_declareted(
    name: &String,
    declarations: &HashSet<IdentifierLeaf>,
) {
    if declarations .iter()
    .find(|p| &p.name == name)
    .is_none() {
        handle_semantic_error(&format!("variable '{}' used before declaration", name));
    }
}

fn check_function_is_declared(
    name: &String,
    declarations: &HashSet<IdentifierLeaf>,
) {
    if declarations .iter()
    .find(|p| &p.name == name)
    .is_none() {
        handle_semantic_error(&format!("function '{}' used before declaration", name));
    }
}

fn check_expression(
    expression: &Expression,
    declared: &HashSet<IdentifierLeaf>,
    scope: &Option<String>,
) {
    match expression {
        Expression::NumberLiteral(_) => {}
        Expression::Identifier(name) => {
            check_variable_is_declareted(name, declared);
        }
        Expression::UnaryOperation { operand, .. } => {
            check_expression(operand, declared, scope);
        }
        Expression::BinOperation {
            left_value,
            right_value,
            ..
        } => {
            check_expression(left_value, declared, scope);
            check_expression(right_value, declared, scope);
        }
        Expression::FunctionCall { name, parameters } => {
            check_function_is_declared(name, declared);
            for expression in parameters {
                check_expression(expression, declared, scope);
            }
        }
    }
}

fn check_command(
    command: &Command,
    declared: &mut HashSet<IdentifierLeaf>,
    scope: &Option<String>,
) {
    match command {
        Command::Attribution {
            variable,
            expression,
        } => {
            let lexema = variable.get_lexema();
            check_variable_is_declareted(lexema, declared);
            check_expression(&expression, declared, scope);
        }
        Command::If {
            condition,
            true_block,
            false_block,
        } => {
            check_expression(&condition, declared, scope);

            let (mut local_declarations, true_block_scope) = create_local_scope(declared, scope, "true_block".to_string());
            check_code_block(true_block, &mut local_declarations, &Some(true_block_scope));

            let (mut local_declarations, false_block_scope) = create_local_scope(declared, scope, "false_block".to_string());
            check_code_block(
                false_block,
                &mut local_declarations,
                &Some(false_block_scope),
            );
        }
        Command::While { condition, block } => {
            check_expression(&condition, declared, scope);

            let (mut local_declarations, while_block_scope) = create_local_scope(declared, scope, "while".to_string());
            check_code_block(block, &mut local_declarations, &Some(while_block_scope));
        }
        Command::Return { expression } => {
            check_expression(expression, declared, scope);
        }
        Command::Print { expression } => {
            check_expression(expression, declared, scope);
        }
        Command::FunctionCall { name, parameters } => {
            check_function_is_declared(name, declared);
            for expression in parameters {
                check_expression(expression, declared, scope);
            }
        }
        Command::Declaration { identifier } => validade_declaration(identifier, declared, scope),
    }
}

fn check_code_block(
    block: &CodeBlock,
    declared: &mut HashSet<IdentifierLeaf>,
    scope: &Option<String>,
) {
    for command in &block.commands {
        check_command(command, declared, scope);
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct IdentifierLeaf {
    name: String,
    scope: Option<String>
}

fn create_local_scope(
    parent_declared: &mut HashSet<IdentifierLeaf>,
    parent_scope: &Option<String>,
    new_scope_name: String,
) -> (HashSet<IdentifierLeaf>, String) {
    let scope_label = parent_scope.clone().unwrap_or(String::from("main"));
    let local_scope = scope_label + ":" + &new_scope_name;

    return (parent_declared.clone(), local_scope);
}

/// - Validates that no variable is used before it has been declared.
fn validade_declaration(
    identifier: &Identifier,
    declared: &mut HashSet<IdentifierLeaf>,
    scope: &Option<String>,
) {
    match identifier {
        Identifier::Variable(variable) => {
            let name = variable.token.get_lexema();

            if declared
                .iter()
                .find(|p| &p.name == name && &p.scope == scope)
                .is_some()
            {
                handle_semantic_error(&format!(
                    "variable '{}' already declared in this scope",
                    name
                ));
            }

            check_expression(&variable.expression, declared, scope);
            declared.insert(IdentifierLeaf {
                name: name.to_string(),
                scope: scope.clone(),
            });
        }
        Identifier::Function(function) => {
            let name = function.token.get_lexema();
            if declared
                .iter()
                .find(|p| &p.name == name && &p.scope == scope)
                .is_some()
            {
                handle_semantic_error(&format!(
                    "function '{}' already declared in this scope",
                    name
                ));
            }

            declared.insert(IdentifierLeaf {
                name: name.to_string(),
                scope: scope.clone(),
            });

            let (mut local_declarations, local_scope) =
                create_local_scope(declared, scope, name.to_string());

            for parameter in &function.parameters {
                let name = parameter.get_lexema();
                local_declarations.insert(IdentifierLeaf {
                    name: name.to_string(),
                    scope: Some(local_scope.clone()),
                });
            }

            check_code_block(
                &function.code_block,
                &mut local_declarations,
                &Some(local_scope.to_string()),
            );
        }
    }
}

pub fn validate_program(program: &Program) {
    let mut declared: HashSet<IdentifierLeaf> = HashSet::new();

    for identifier in &program.declarations {
        validade_declaration(identifier, &mut declared, &None);
    }

    for command in &program.commands {
        check_command(command, &mut declared, &None);
    }

    let Some(Command::Return { expression }) = program.commands.last() else {
        handle_semantic_error("main block must have return expression for last command")
    };
    check_expression(expression, &declared, &None)
}
