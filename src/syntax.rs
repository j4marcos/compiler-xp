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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

fn get_operator(tokens: &mut VecDeque<Token>, valid_operators: &[Operator]) -> Option<Operator> {
    let Some(token) = tokens.front() else {
        return None;
    };

    let operator = match token.class {
        TokenClass::Space => {
            tokens.pop_front();
            return get_operator(tokens, valid_operators);
        }
        TokenClass::SubOperator => Operator::Sub,
        TokenClass::SumOperator => Operator::Sum,
        TokenClass::DivOperator => Operator::Div,
        TokenClass::MulOperator => Operator::Mul,
        _ => return None,
    };

    if valid_operators.len() == 0 {
        return Some(operator);
    } else if valid_operators.contains(&operator) {
        return Some(operator);
    } else {
        return None;
    }
}

fn get_expression_multiplication(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression(tokens);

    while let Some(next_operator) = get_operator(tokens, &[Operator::Mul, Operator::Div]) {
        tokens.pop_front(); // pop operator
        let right = get_expression(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator: next_operator,
            right_value: Box::new(right),
        }
    }

    return left;
}

fn get_expression_addition(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression_multiplication(tokens);

    while let Some(next_operator) = get_operator(tokens, &[Operator::Sub, Operator::Sum]) {
        tokens.pop_front(); // pop operator
        let right = get_expression_multiplication(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator: next_operator,
            right_value: Box::new(right),
        }
    }

    return left;
}

fn pop_right_parenteses(tokens: &mut VecDeque<Token>) {
    match tokens.pop_front() {
        Some(token) => match token.class {
            TokenClass::Space => pop_right_parenteses(tokens),
            TokenClass::RightParentheses => return,
            _ => handle_sintax_error("expecting a closing parenteses"),
        },
        None => return,
    }
}

fn get_expression(tokens: &mut VecDeque<Token>) -> Expression {
    let token = tokens.pop_front();

    return match token {
        None => handle_sintax_error("Miss token to complete expression"),
        Some(token) => match token.class {
            TokenClass::Space => get_expression(tokens),
            TokenClass::Number => {
                Expression::NumberLiteral(token.get_value().expect("invalid number token"))
            }
            TokenClass::LeftParentheses => {
                let innner_expression = get_expression_addition(tokens);
                pop_right_parenteses(tokens);
                return innner_expression;
            }
            _ => handle_sintax_error("Expecting a expression"),
        },
    };
}

pub fn extract_expression(tokens_list: TokenList) -> Expression {
    let mut tokens_queue = VecDeque::from(tokens_list.get_tokens());
    let result = get_expression_addition(&mut tokens_queue);
    return result;
}
