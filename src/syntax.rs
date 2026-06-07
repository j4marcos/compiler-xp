use crate::lexical::*;
use std::collections::VecDeque;

pub enum Expression {
    NumberLiteral(i32),
    BinOperation {
        left_value: Box<Expression>,
        operator: Operator,
        right_value: Box<Expression>,
    },
}

pub enum Operator {
    Sum,
    Sub,
    Div,
    Mul,
}

impl Expression {
    fn print_tree(&self, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        println!();

        match self {
            Expression::NumberLiteral(n) => {
                println!("{}{}Number({})", prefix, connector, n);
            }
            Expression::BinOperation {
                left_value,
                operator,
                right_value,
            } => {
                println!("{}{}BinOp({})", prefix, connector, operator);
                left_value.print_tree(&child_prefix, false);
                right_value.print_tree(&child_prefix, true);
            }
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Sum => write!(f, "Sum"),
            Operator::Sub => write!(f, "Sub"),
            Operator::Div => write!(f, "Div"),
            Operator::Mul => write!(f, "Mul"),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print_tree("", true);
        Ok(())
    }
}

fn handle_sintax_error(reason: &str) -> ! {
    eprintln!("Error sintax: {}", reason);

    std::process::exit(1);
}

fn get_operator(tokens: &mut VecDeque<Token>) -> Operator {
    let Some(token) = tokens.pop_front() else {
        handle_sintax_error("Must be binary expression");
    };

    match token.class {
        TokenClass::Space => get_operator(tokens),
        TokenClass::SubOperator => Operator::Sub,
        TokenClass::SumOperator => Operator::Sum,
        TokenClass::DivOperator => Operator::Div,
        TokenClass::MulOperator => Operator::Mul,
        _ => {
            println!("token at: {:?}", token);
            handle_sintax_error("Expect a operator in the expression")
        }
    }
}

fn analyse_expression(tokens: &mut VecDeque<Token>) -> Expression {
    let token = tokens.pop_front();

    return match token {
        None => handle_sintax_error("Miss token to complete expression"),
        Some(token) => match token.class {
            TokenClass::Space => analyse_expression(tokens),
            TokenClass::Number => {
                Expression::NumberLiteral(token.get_value().expect("invalid number token"))
            }
            TokenClass::LeftParentheses => {
                let expression = Expression::BinOperation {
                    left_value: Box::new(analyse_expression(tokens)),
                    operator: get_operator(tokens),
                    right_value: Box::new(analyse_expression(tokens)),
                };
                tokens.pop_front();
                return expression;
            }
            _ => handle_sintax_error("Expecting a expression"),
        },
    };
}

pub fn extract_expression(tokens_list: TokenList) -> Expression {
    let mut tokens_queue = VecDeque::from(tokens_list.get_tokens());
    return analyse_expression(&mut tokens_queue);
}
