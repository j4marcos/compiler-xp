use crate::lexical;
use crate::syntax::{
    AssignTarget, CodeBlock, Command, Expression, Function, Identifier, Import, Program, Variable,
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn handle_resolve_error(reason: &str) -> ! {
    eprintln!("Error resolve: {}", reason);
    std::process::exit(1);
}

fn qualify(alias: &str, name: &str) -> String {
    format!("{}::{}", alias, name)
}

fn rename_token_name(token: &mut crate::lexical::Token, alias: &str, globals: &HashSet<String>) {
    let name = token.get_lexema().clone();
    if globals.contains(&name) {
        token.lexema = qualify(alias, &name);
    }
}

fn rewrite_name(name: &mut String, alias: &str, globals: &HashSet<String>) {
    if globals.contains(name) {
        *name = qualify(alias, name);
    }
}

fn rewrite_expression(expr: &mut Expression, alias: &str, globals: &HashSet<String>) {
    match expr {
        Expression::NumberLiteral(_) | Expression::TextLiteral(_) => {}
        Expression::Identifier(name) => rewrite_name(name, alias, globals),
        Expression::UnaryOperation { operand, .. } => {
            rewrite_expression(operand, alias, globals);
        }
        Expression::BinOperation {
            left_value,
            right_value,
            ..
        } => {
            rewrite_expression(left_value, alias, globals);
            rewrite_expression(right_value, alias, globals);
        }
        Expression::FunctionCall { name, parameters } => {
            rewrite_name(name, alias, globals);
            for p in parameters {
                rewrite_expression(p, alias, globals);
            }
        }
        Expression::Index { array, index } => {
            rewrite_name(array, alias, globals);
            rewrite_expression(index, alias, globals);
        }
        Expression::ArrayLiteral(elements) => {
            for e in elements {
                rewrite_expression(e, alias, globals);
            }
        }
    }
}

fn rewrite_command(cmd: &mut Command, alias: &str, globals: &HashSet<String>) {
    match cmd {
        Command::If {
            condition,
            true_block,
            false_block,
        } => {
            rewrite_expression(condition, alias, globals);
            rewrite_block(true_block, alias, globals);
            rewrite_block(false_block, alias, globals);
        }
        Command::While { condition, block } => {
            rewrite_expression(condition, alias, globals);
            rewrite_block(block, alias, globals);
        }
        Command::Attribution { target, expression } => {
            match target {
                AssignTarget::Variable(tok) => rename_token_name(tok, alias, globals),
                AssignTarget::Index { array, index } => {
                    rename_token_name(array, alias, globals);
                    rewrite_expression(index, alias, globals);
                }
            }
            rewrite_expression(expression, alias, globals);
        }
        Command::FunctionCall { name, parameters } => {
            rewrite_name(name, alias, globals);
            for p in parameters {
                rewrite_expression(p, alias, globals);
            }
        }
        Command::Return { expression } | Command::Print { expression } => {
            rewrite_expression(expression, alias, globals);
        }
        Command::Declaration { identifier } => match identifier {
            Identifier::Variable(Variable {
                name, expression, ..
            }) => {
                // locals keep their names; only rewrite RHS refs to globals
                let _ = name;
                rewrite_expression(expression, alias, globals);
            }
            Identifier::Function(_) => {
                handle_resolve_error("nested function declarations are not supported in libraries");
            }
        },
    }
}

fn rewrite_block(block: &mut CodeBlock, alias: &str, globals: &HashSet<String>) {
    for cmd in &mut block.commands {
        rewrite_command(cmd, alias, globals);
    }
}

fn rewrite_function(function: &mut Function, alias: &str, globals: &HashSet<String>) {
    rewrite_block(&mut function.code_block, alias, globals);
}

fn qualify_library(program: &mut Program, alias: &str) {
    if !program.imports.is_empty() {
        handle_resolve_error(&format!(
            "library '{}' cannot contain nested imports",
            alias
        ));
    }

    let mut globals: HashSet<String> = HashSet::new();
    for decl in &program.declarations {
        match decl {
            Identifier::Function(f) => {
                let name = f.name.get_lexema().clone();
                if name == "main" {
                    handle_resolve_error(&format!(
                        "library '{}' cannot define main",
                        alias
                    ));
                }
                globals.insert(name);
            }
            Identifier::Variable(v) => {
                globals.insert(v.name.get_lexema().clone());
            }
        }
    }

    for decl in &mut program.declarations {
        match decl {
            Identifier::Function(f) => {
                rewrite_function(f, alias, &globals);
                let name = f.name.get_lexema().clone();
                f.name.lexema = qualify(alias, &name);
            }
            Identifier::Variable(v) => {
                rewrite_expression(&mut v.expression, alias, &globals);
                let name = v.name.get_lexema().clone();
                v.name.lexema = qualify(alias, &name);
            }
        }
    }
}

fn resolve_import_path(source_dir: &Path, import_path: &str) -> PathBuf {
    let path = Path::new(import_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        source_dir.join(path)
    }
}

fn load_library(alias: &str, path: &Path) -> Program {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        handle_resolve_error(&format!(
            "cannot read library '{}' at '{}': {}",
            alias,
            path.display(),
            e
        ))
    });
    let tokens = lexical::extract_tokens(&source);
    let mut program = crate::syntax::build_program(tokens);
    qualify_library(&mut program, alias);
    program
}

/// Resolve imports relative to `source_path`, merge library decls into the program,
/// and clear `imports` on the resulting program.
pub fn resolve_imports(mut program: Program, source_path: &Path) -> Program {
    if program.imports.is_empty() {
        return program;
    }

    let source_dir = source_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let mut seen_aliases: HashSet<String> = HashSet::new();
    let mut lib_declarations: Vec<Identifier> = Vec::new();

    for Import { alias, path } in &program.imports {
        if !seen_aliases.insert(alias.clone()) {
            handle_resolve_error(&format!("duplicate import alias '{}'", alias));
        }
        let full_path = resolve_import_path(source_dir, path);
        let lib = load_library(alias, &full_path);
        lib_declarations.extend(lib.declarations);
    }

    let mut declarations = lib_declarations;
    declarations.append(&mut program.declarations);

    Program {
        imports: Vec::new(),
        declarations,
    }
}
