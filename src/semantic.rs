use crate::syntax::{
    AssignTarget, CodeBlock, Command, Expression, Identifier, Parameter, Program, Type,
};
use std::collections::{HashMap, HashSet};

fn handle_semantic_error(reason: &str) -> ! {
    eprintln!("Error semantic: {}", reason);
    std::process::exit(1);
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct IdentifierLeaf {
    name: String,
    scope: Option<String>,
    typ: Type,
}

struct FunctionInfo {
    params: Vec<Type>,
    return_type: Type,
}

fn find_by_name<'a>(
    name: &String,
    declarations: &'a HashSet<IdentifierLeaf>,
) -> Option<&'a IdentifierLeaf> {
    declarations.iter().find(|p| &p.name == name)
}

fn check_variable_is_declareted(name: &String, declarations: &HashSet<IdentifierLeaf>) {
    if find_by_name(name, declarations).is_none() {
        handle_semantic_error(&format!("variable '{}' used before declaration", name));
    }
}

fn builtin_return_type(name: &str, arg_types: &[Type]) -> Option<Type> {
    match name {
        "push" => {
            if arg_types.len() == 2
                && arg_types[0].is_array_like()
                && matches!(arg_types[1], Type::Num | Type::Bool)
            {
                Some(arg_types[0])
            } else {
                None
            }
        }
        "len" => {
            if arg_types.len() == 1 && arg_types[0].is_array_like() {
                Some(Type::Num)
            } else {
                None
            }
        }
        "pop" => {
            if arg_types.len() == 1 && arg_types[0].is_array_like() {
                Some(Type::Num)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn expression_type(
    expression: &Expression,
    declared: &HashSet<IdentifierLeaf>,
    functions: &HashMap<String, FunctionInfo>,
) -> Type {
    match expression {
        Expression::NumberLiteral(_) => Type::Num,
        Expression::TextLiteral(_) => Type::Text,
        Expression::Identifier(name) => find_by_name(name, declared)
            .map(|l| l.typ)
            .unwrap_or(Type::Num),
        Expression::UnaryOperation { operator, operand } => {
            let _ = operand;
            match operator {
                crate::syntax::Operator::Not => Type::Bool,
                _ => Type::Num,
            }
        }
        Expression::BinOperation { operator, .. } => match operator {
            crate::syntax::Operator::And
            | crate::syntax::Operator::Or
            | crate::syntax::Operator::Equal
            | crate::syntax::Operator::LessThan
            | crate::syntax::Operator::LessEqual
            | crate::syntax::Operator::GreaterThan
            | crate::syntax::Operator::GreaterEqual => Type::Bool,
            _ => Type::Num,
        },
        Expression::FunctionCall { name, parameters } => {
            let arg_types: Vec<Type> = parameters
                .iter()
                .map(|p| expression_type(p, declared, functions))
                .collect();
            if let Some(t) = builtin_return_type(name, &arg_types) {
                return t;
            }
            functions
                .get(name)
                .map(|f| f.return_type)
                .unwrap_or(Type::Num)
        }
        Expression::Index { .. } => Type::Num,
        Expression::ArrayLiteral(_) => Type::List,
    }
}

fn types_compatible(expected: Type, actual: Type) -> bool {
    expected == actual
        || (expected == Type::Bool && matches!(actual, Type::Num | Type::Bool))
        || (expected == Type::Num && actual == Type::Bool)
}

fn check_expression(
    expression: &Expression,
    declared: &HashSet<IdentifierLeaf>,
    functions: &HashMap<String, FunctionInfo>,
    scope: &Option<String>,
) {
    match expression {
        Expression::NumberLiteral(_) | Expression::TextLiteral(_) => {}
        Expression::Identifier(name) => {
            check_variable_is_declareted(name, declared);
        }
        Expression::UnaryOperation { operand, .. } => {
            check_expression(operand, declared, functions, scope);
        }
        Expression::BinOperation {
            left_value,
            right_value,
            ..
        } => {
            check_expression(left_value, declared, functions, scope);
            check_expression(right_value, declared, functions, scope);
        }
        Expression::FunctionCall { name, parameters } => {
            for expression in parameters {
                check_expression(expression, declared, functions, scope);
            }
            let arg_types: Vec<Type> = parameters
                .iter()
                .map(|p| expression_type(p, declared, functions))
                .collect();

            if builtin_return_type(name, &arg_types).is_some() {
                return;
            }

            if name == "push" || name == "len" || name == "pop" {
                handle_semantic_error(&format!(
                    "invalid arguments for builtin '{}'",
                    name
                ));
            }

            let Some(info) = functions.get(name) else {
                handle_semantic_error(&format!("function '{}' used before declaration", name));
            };

            if info.params.len() != arg_types.len() {
                handle_semantic_error(&format!(
                    "function '{}' expects {} args, got {}",
                    name,
                    info.params.len(),
                    arg_types.len()
                ));
            }

            for (i, (expected, actual)) in info.params.iter().zip(arg_types.iter()).enumerate() {
                if !types_compatible(*expected, *actual) {
                    handle_semantic_error(&format!(
                        "argument {} of '{}': expected {:?}, got {:?}",
                        i + 1,
                        name,
                        expected,
                        actual
                    ));
                }
            }
        }
        Expression::Index { array, index } => {
            check_variable_is_declareted(array, declared);
            let arr_t = find_by_name(array, declared).map(|l| l.typ);
            if !matches!(arr_t, Some(Type::List) | Some(Type::Text)) {
                handle_semantic_error(&format!("'{}' is not a list/text", array));
            }
            check_expression(index, declared, functions, scope);
        }
        Expression::ArrayLiteral(elements) => {
            for element in elements {
                check_expression(element, declared, functions, scope);
            }
        }
    }
}

fn check_command(
    command: &Command,
    declared: &mut HashSet<IdentifierLeaf>,
    functions: &mut HashMap<String, FunctionInfo>,
    scope: &Option<String>,
    expected_return: Option<Type>,
) {
    match command {
        Command::Attribution { target, expression } => {
            check_expression(expression, declared, functions, scope);
            match target {
                AssignTarget::Variable(variable) => {
                    let lexema = variable.get_lexema();
                    check_variable_is_declareted(lexema, declared);
                    let left = find_by_name(lexema, declared).map(|l| l.typ).unwrap();
                    let right = expression_type(expression, declared, functions);
                    if !types_compatible(left, right) {
                        handle_semantic_error(&format!(
                            "cannot assign {:?} to '{}' ({:?})",
                            right, lexema, left
                        ));
                    }
                }
                AssignTarget::Index { array, index } => {
                    check_variable_is_declareted(array.get_lexema(), declared);
                    let arr_t = find_by_name(array.get_lexema(), declared).map(|l| l.typ);
                    if !matches!(arr_t, Some(Type::List) | Some(Type::Text)) {
                        handle_semantic_error(&format!(
                            "'{}' is not a list/text",
                            array.get_lexema()
                        ));
                    }
                    check_expression(index, declared, functions, scope);
                    let right = expression_type(expression, declared, functions);
                    if !types_compatible(Type::Num, right) {
                        handle_semantic_error("indexed assignment expects num");
                    }
                }
            }
        }
        Command::If {
            condition,
            true_block,
            false_block,
        } => {
            check_expression(condition, declared, functions, scope);
            let (mut local_declarations, true_block_scope) =
                create_local_scope(declared, scope, "true_block".to_string());
            check_code_block(
                true_block,
                &mut local_declarations,
                functions,
                &Some(true_block_scope),
                expected_return,
            );

            let (mut local_declarations, false_block_scope) =
                create_local_scope(declared, scope, "false_block".to_string());
            check_code_block(
                false_block,
                &mut local_declarations,
                functions,
                &Some(false_block_scope),
                expected_return,
            );
        }
        Command::While { condition, block } => {
            check_expression(condition, declared, functions, scope);
            let (mut local_declarations, while_block_scope) =
                create_local_scope(declared, scope, "while".to_string());
            check_code_block(
                block,
                &mut local_declarations,
                functions,
                &Some(while_block_scope),
                expected_return,
            );
        }
        Command::Return { expression } => {
            check_expression(expression, declared, functions, scope);
            if let Some(expected) = expected_return {
                let actual = expression_type(expression, declared, functions);
                if !types_compatible(expected, actual) {
                    handle_semantic_error(&format!(
                        "return type {:?}, expected {:?}",
                        actual, expected
                    ));
                }
            }
        }
        Command::Print { expression } => {
            check_expression(expression, declared, functions, scope);
        }
        Command::FunctionCall { name, parameters } => {
            let call = Expression::FunctionCall {
                name: name.clone(),
                parameters: parameters.clone(),
            };
            check_expression(&call, declared, functions, scope);
        }
        Command::Declaration { identifier } => {
            validade_declaration(identifier, declared, functions, scope)
        }
    }
}

fn check_code_block(
    block: &CodeBlock,
    declared: &mut HashSet<IdentifierLeaf>,
    functions: &mut HashMap<String, FunctionInfo>,
    scope: &Option<String>,
    expected_return: Option<Type>,
) {
    for command in &block.commands {
        check_command(command, declared, functions, scope, expected_return);
    }
}

fn create_local_scope(
    parent_declared: &mut HashSet<IdentifierLeaf>,
    parent_scope: &Option<String>,
    new_scope_name: String,
) -> (HashSet<IdentifierLeaf>, String) {
    let scope_label = parent_scope.clone().unwrap_or(String::from("main"));
    let local_scope = scope_label + ":" + &new_scope_name;
    (parent_declared.clone(), local_scope)
}

fn validade_declaration(
    identifier: &Identifier,
    declared: &mut HashSet<IdentifierLeaf>,
    functions: &mut HashMap<String, FunctionInfo>,
    scope: &Option<String>,
) {
    match identifier {
        Identifier::Variable(variable) => {
            let name = variable.name.get_lexema();
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

            check_expression(&variable.expression, declared, functions, scope);
            let right = expression_type(&variable.expression, declared, functions);
            if !types_compatible(variable.typ, right) {
                handle_semantic_error(&format!(
                    "cannot initialize '{}' ({:?}) with {:?}",
                    name, variable.typ, right
                ));
            }

            declared.insert(IdentifierLeaf {
                name: name.to_string(),
                scope: scope.clone(),
                typ: variable.typ,
            });
        }
        Identifier::Function(function) => {
            let name = function.name.get_lexema();
            if declared
                .iter()
                .find(|p| &p.name == name && &p.scope == scope)
                .is_some()
                || functions.contains_key(name)
            {
                handle_semantic_error(&format!(
                    "function '{}' already declared in this scope",
                    name
                ));
            }

            let params: Vec<Type> = function.parameters.iter().map(|p| p.typ).collect();
            functions.insert(
                name.to_string(),
                FunctionInfo {
                    params: params.clone(),
                    return_type: function.return_type,
                },
            );

            let (mut local_declarations, local_scope) =
                create_local_scope(declared, scope, name.to_string());

            for Parameter { name, typ } in &function.parameters {
                local_declarations.insert(IdentifierLeaf {
                    name: name.get_lexema().to_string(),
                    scope: Some(local_scope.clone()),
                    typ: *typ,
                });
            }

            check_code_block(
                &function.code_block,
                &mut local_declarations,
                functions,
                &Some(local_scope.to_string()),
                Some(function.return_type),
            );
        }
    }
}

pub fn validate_program(program: &Program) {
    let mut declared: HashSet<IdentifierLeaf> = HashSet::new();
    let mut functions: HashMap<String, FunctionInfo> = HashMap::new();

    for identifier in &program.declarations {
        validade_declaration(identifier, &mut declared, &mut functions, &None);
    }

    let Some(Identifier::Function(main)) = program
        .declarations
        .iter()
        .find(|d| matches!(d, Identifier::Function(f) if f.name.get_lexema() == "main"))
    else {
        handle_semantic_error("program must have a main function");
    };

    if !matches!(main.code_block.commands.last(), Some(Command::Return { .. })) {
        handle_semantic_error("main block must have return expression for last command");
    }

    if !matches!(main.return_type, Type::Num | Type::Bool) {
        handle_semantic_error("main must return num/bool");
    }
}
