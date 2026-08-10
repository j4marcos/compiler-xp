use crate::lexical::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Num,
    List,
    Bool,
    Text,
}

impl Type {
    pub fn is_array_like(self) -> bool {
        matches!(self, Type::List | Type::Text)
    }

    pub fn from_keyword(keyword: KeyWord) -> Option<Type> {
        match keyword {
            KeyWord::Num => Some(Type::Num),
            KeyWord::List => Some(Type::List),
            KeyWord::Bool => Some(Type::Bool),
            KeyWord::Text => Some(Type::Text),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    NumberLiteral(i32),
    Identifier(String),
    UnaryOperation {
        operator: Operator,
        operand: Box<Expression>,
    },
    BinOperation {
        left_value: Box<Expression>,
        operator: Operator,
        right_value: Box<Expression>,
    },
    FunctionCall {
        name: String,
        parameters: Vec<Expression>,
    },
    Index {
        array: String,
        index: Box<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    TextLiteral(String),
}

#[derive(Clone, Debug)]
pub enum AssignTarget {
    Variable(Token),
    Index { array: Token, index: Expression },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Operator {
    Sum,
    Sub,
    Div,
    Mul,
    Mod,
    Equal,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Not,
    Or,
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
            Expression::Identifier(name) => {
                println!("{}{}Ident({})", prefix, connector, name);
            }
            Expression::UnaryOperation { operator, operand } => {
                println!("{}{}UnaryOp({})", prefix, connector, operator);
                operand.print_tree(&child_prefix, true);
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
            Expression::FunctionCall { name, parameters } => {
                println!("{}{}Call({})", prefix, connector, name);
                let last = parameters.len().saturating_sub(1);
                for (i, param) in parameters.iter().enumerate() {
                    param.print_tree(&child_prefix, i == last);
                }
            }
            Expression::Index { array, index } => {
                println!("{}{}Index({})", prefix, connector, array);
                index.print_tree(&child_prefix, true);
            }
            Expression::ArrayLiteral(elements) => {
                println!("{}{}ArrayLiteral", prefix, connector);
                let last = elements.len().saturating_sub(1);
                for (i, elem) in elements.iter().enumerate() {
                    elem.print_tree(&child_prefix, i == last);
                }
            }
            Expression::TextLiteral(s) => {
                println!("{}{}Text(\"{}\")", prefix, connector, s);
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
            Operator::Mod => write!(f, "Mod"),
            Operator::Equal => write!(f, "Equal"),
            Operator::GreaterThan => write!(f, ">"),
            Operator::GreaterEqual => write!(f, ">="),
            Operator::LessThan => write!(f, "<"),
            Operator::LessEqual => write!(f, "<="),
            Operator::And => write!(f, "and"),
            Operator::Not => write!(f, "not"),
            Operator::Or => write!(f, "or"),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print_tree("", true);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
    pub typ: Type,
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Token,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub code_block: CodeBlock,
}

#[derive(Debug, Clone)]
pub enum Identifier {
    Variable(Variable),
    Function(Function),
}

#[derive(Debug)]
pub struct Program {
    pub declarations: Vec<Identifier>,
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub enum Command {
    If {
        condition: Expression,
        true_block: CodeBlock,
        false_block: CodeBlock,
    },
    While {
        condition: Expression,
        block: CodeBlock,
    },
    Attribution {
        target: AssignTarget,
        expression: Expression,
    },
    FunctionCall {
        name: String,
        parameters: Vec<Expression>,
    },
    Return {
        expression: Expression,
    },
    Print {
        expression: Expression,
    },
    Declaration {
        identifier: Identifier,
    },
}

fn handle_sintax_error(reason: &str) -> ! {
    eprintln!("Error sintax: {}", reason);
    std::process::exit(1);
}

fn skip_whitespace(tokens: &mut VecDeque<Token>) {
    while let Some(token) = tokens.front() {
        match token.class {
            TokenClass::Space | TokenClass::NewLine => {
                tokens.pop_front();
            }
            _ => break,
        }
    }
}

fn is_keyword(token: &Token, keyword: KeyWord) -> bool {
    token.class == TokenClass::KeyWord && KeyWord::from_lexema(token.get_lexema()) == Some(keyword)
}

fn consume_token_class(tokens: &mut VecDeque<Token>, class: TokenClass) {
    skip_whitespace(tokens);
    match tokens.pop_front() {
        Some(token) if token.class == class => {}
        _ => handle_sintax_error(&format!("expecting {:?} here", class)),
    }
}

fn next_token_class(tokens: &VecDeque<Token>, read_index: usize) -> Option<TokenClass> {
    let Some(next_token) = tokens.get(read_index) else {
        return None;
    };
    match next_token.class {
        TokenClass::Space | TokenClass::NewLine => next_token_class(tokens, read_index + 1),
        _ => Some(next_token.class),
    }
}

fn parse_type(tokens: &mut VecDeque<Token>) -> Type {
    skip_whitespace(tokens);
    let Some(token) = tokens.pop_front() else {
        handle_sintax_error("expected a type")
    };
    if token.class != TokenClass::KeyWord {
        handle_sintax_error("expected a type keyword");
    }
    match KeyWord::from_lexema(token.get_lexema()).and_then(Type::from_keyword) {
        Some(t) => t,
        None => handle_sintax_error(&format!("'{}' is not a type", token.get_lexema())),
    }
}

fn get_operator(tokens: &mut VecDeque<Token>, valid_operators: &[Operator]) -> Option<Operator> {
    skip_whitespace(tokens);

    let Some(token) = tokens.front() else {
        return None;
    };

    let operator = match token.class {
        TokenClass::SubOperator => Operator::Sub,
        TokenClass::SumOperator => Operator::Sum,
        TokenClass::DivOperator => Operator::Div,
        TokenClass::MulOperator => Operator::Mul,
        TokenClass::ModOperator => Operator::Mod,
        TokenClass::EqualOperator => Operator::Equal,
        TokenClass::GreaterThanOperator => Operator::GreaterThan,
        TokenClass::GreaterEqualOperator => Operator::GreaterEqual,
        TokenClass::LessThanOperator => Operator::LessThan,
        TokenClass::LessEqualOperator => Operator::LessEqual,
        TokenClass::AndOperator => Operator::And,
        TokenClass::OrOperator => Operator::Or,
        _ => return None,
    };

    if valid_operators.is_empty() || valid_operators.contains(&operator) {
        Some(operator)
    } else {
        None
    }
}

fn get_expression_multiplication(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression_unary(tokens);

    while let Some(next_operator) =
        get_operator(tokens, &[Operator::Mul, Operator::Div, Operator::Mod])
    {
        tokens.pop_front();
        let right = get_expression_unary(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator: next_operator,
            right_value: Box::new(right),
        }
    }

    left
}

fn get_expression_addition(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression_multiplication(tokens);

    while let Some(next_operator) = get_operator(tokens, &[Operator::Sub, Operator::Sum]) {
        tokens.pop_front();
        let right = get_expression_multiplication(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator: next_operator,
            right_value: Box::new(right),
        }
    }

    left
}

fn get_expression_comparation(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression_addition(tokens);

    while let Some(next_operator) = get_operator(
        tokens,
        &[
            Operator::Equal,
            Operator::GreaterThan,
            Operator::GreaterEqual,
            Operator::LessThan,
            Operator::LessEqual,
        ],
    ) {
        tokens.pop_front();
        let right = get_expression_addition(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator: next_operator,
            right_value: Box::new(right),
        }
    }

    left
}

fn get_expression_and(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression_comparation(tokens);

    while let Some(operator) = get_operator(tokens, &[Operator::And]) {
        tokens.pop_front();
        let right = get_expression_comparation(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator,
            right_value: Box::new(right),
        };
    }

    left
}

fn get_expression_or(tokens: &mut VecDeque<Token>) -> Expression {
    let mut left = get_expression_and(tokens);

    while let Some(operator) = get_operator(tokens, &[Operator::Or]) {
        tokens.pop_front();
        let right = get_expression_and(tokens);
        left = Expression::BinOperation {
            left_value: Box::new(left),
            operator,
            right_value: Box::new(right),
        };
    }

    left
}

fn apply_method_calls(mut expr: Expression, tokens: &mut VecDeque<Token>) -> Expression {
    loop {
        skip_whitespace(tokens);
        if next_token_class(tokens, 0) != Some(TokenClass::Dot) {
            break;
        }
        consume_token_class(tokens, TokenClass::Dot);
        skip_whitespace(tokens);
        let Some(method) = tokens.pop_front() else {
            handle_sintax_error("expected method name after '.'")
        };
        if method.class != TokenClass::Identifier {
            handle_sintax_error("expected method name after '.'");
        }
        let method_name = method.get_lexema().to_string();

        // `.len` / `.pop` builtins (parens optional)
        if (method_name == "len" || method_name == "pop")
            && next_token_class(tokens, 0) != Some(TokenClass::LeftParentheses)
        {
            expr = Expression::FunctionCall {
                name: method_name,
                parameters: vec![expr],
            };
            continue;
        }

        consume_token_class(tokens, TokenClass::LeftParentheses);
        let mut parameters: Vec<Expression> = vec![expr];
        read_call_parameters(tokens, &mut parameters);
        expr = Expression::FunctionCall {
            name: method_name,
            parameters,
        };
    }
    expr
}

fn get_expression_unary(tokens: &mut VecDeque<Token>) -> Expression {
    skip_whitespace(tokens);

    if tokens
        .front()
        .is_some_and(|token| token.class == TokenClass::NotOperator)
    {
        tokens.pop_front();
        return Expression::UnaryOperation {
            operator: Operator::Not,
            operand: Box::new(get_expression_unary(tokens)),
        };
    }

    if tokens
        .front()
        .is_some_and(|token| token.class == TokenClass::SubOperator)
    {
        tokens.pop_front();
        return Expression::UnaryOperation {
            operator: Operator::Sub,
            operand: Box::new(get_expression_unary(tokens)),
        };
    }

    let primary = get_primary(tokens);
    apply_method_calls(primary, tokens)
}

fn get_primary(tokens: &mut VecDeque<Token>) -> Expression {
    skip_whitespace(tokens);

    match tokens.pop_front() {
        None => handle_sintax_error("missing token to complete expression"),
        Some(token) => match token.class {
            TokenClass::Number => {
                Expression::NumberLiteral(token.get_number().expect("invalid number token"))
            }
            TokenClass::StringLiteral => Expression::TextLiteral(token.get_lexema().clone()),
            TokenClass::Identifier => {
                let lexema = token.get_lexema().to_string();
                match next_token_class(tokens, 0) {
                    Some(TokenClass::LeftParentheses) => {
                        consume_token_class(tokens, TokenClass::LeftParentheses);
                        let mut parameters: Vec<Expression> = Vec::new();
                        read_call_parameters(tokens, &mut parameters);
                        Expression::FunctionCall {
                            name: lexema,
                            parameters,
                        }
                    }
                    Some(TokenClass::LeftBracket) => {
                        consume_token_class(tokens, TokenClass::LeftBracket);
                        let index = extract_expression(tokens);
                        consume_token_class(tokens, TokenClass::RightBracket);
                        Expression::Index {
                            array: lexema,
                            index: Box::new(index),
                        }
                    }
                    _ => Expression::Identifier(lexema),
                }
            }
            TokenClass::KeyWord => match KeyWord::from_lexema(token.get_lexema()) {
                Some(KeyWord::True) => Expression::NumberLiteral(1),
                Some(KeyWord::False) => Expression::NumberLiteral(0),
                Some(KeyWord::Main) => {
                    let lexema = token.get_lexema().to_string();
                    match next_token_class(tokens, 0) {
                        Some(TokenClass::LeftParentheses) => {
                            consume_token_class(tokens, TokenClass::LeftParentheses);
                            let mut parameters: Vec<Expression> = Vec::new();
                            read_call_parameters(tokens, &mut parameters);
                            Expression::FunctionCall {
                                name: lexema,
                                parameters,
                            }
                        }
                        _ => Expression::Identifier(lexema),
                    }
                }
                _ => handle_sintax_error(&format!(
                    "unexpected keyword '{}' in expression",
                    token.get_lexema()
                )),
            },
            TokenClass::LeftBracket => {
                let mut elements: Vec<Expression> = Vec::new();
                skip_whitespace(tokens);
                if next_token_class(tokens, 0) != Some(TokenClass::RightBracket) {
                    loop {
                        elements.push(extract_expression(tokens));
                        skip_whitespace(tokens);
                        match next_token_class(tokens, 0) {
                            Some(TokenClass::Comma) => {
                                consume_token_class(tokens, TokenClass::Comma);
                            }
                            Some(TokenClass::RightBracket) => break,
                            _ => handle_sintax_error("expected ',' or ']' in array literal"),
                        }
                    }
                }
                consume_token_class(tokens, TokenClass::RightBracket);
                Expression::ArrayLiteral(elements)
            }
            TokenClass::LeftParentheses => {
                let inner_expression = extract_expression(tokens);
                consume_token_class(tokens, TokenClass::RightParentheses);
                inner_expression
            }
            _ => handle_sintax_error("expecting a number, identifier, array literal or '('"),
        },
    }
}

fn extract_expression(tokens: &mut VecDeque<Token>) -> Expression {
    get_expression_or(tokens)
}

fn extract_variable_declaration(tokens: &mut VecDeque<Token>) -> Identifier {
    let typ = parse_type(tokens);
    skip_whitespace(tokens);
    let name = tokens
        .pop_front()
        .expect("variable declaration must have a identifier")
        .clone();
    if name.class != TokenClass::Identifier {
        handle_sintax_error("variable name must have an identifier");
    }
    consume_token_class(tokens, TokenClass::Attribution);
    let expression = extract_expression(tokens);
    consume_token_class(tokens, TokenClass::Semicolon);
    Identifier::Variable(Variable {
        name,
        typ,
        expression,
    })
}

fn read_declaration_parameter(tokens: &mut VecDeque<Token>) -> Parameter {
    let typ = parse_type(tokens);
    skip_whitespace(tokens);
    let Some(token) = tokens.pop_front() else {
        handle_sintax_error("missing parameter name in declaration");
    };
    if token.class != TokenClass::Identifier {
        handle_sintax_error("parameter must be an identifier");
    }
    Parameter { name: token, typ }
}

fn read_declaration_parameters(tokens: &mut VecDeque<Token>, parameters: &mut Vec<Parameter>) {
    skip_whitespace(tokens);

    match tokens.front() {
        Some(token) => match token.class {
            TokenClass::RightParentheses => return,
            TokenClass::Comma => {
                consume_token_class(tokens, TokenClass::Comma);
                parameters.push(read_declaration_parameter(tokens));
                read_declaration_parameters(tokens, parameters);
            }
            _ => {
                parameters.push(read_declaration_parameter(tokens));
                read_declaration_parameters(tokens, parameters);
            }
        },
        None => handle_sintax_error("function call incomplete"),
    }
}

fn extract_function_declaration(tokens: &mut VecDeque<Token>) -> Identifier {
    consume_token_class(tokens, TokenClass::KeyWord);
    skip_whitespace(tokens);
    let name = tokens
        .pop_front()
        .expect("function declaration must have a identifier")
        .clone();
    consume_token_class(tokens, TokenClass::LeftParentheses);
    let mut parameters: Vec<Parameter> = Vec::new();
    read_declaration_parameters(tokens, &mut parameters);
    consume_token_class(tokens, TokenClass::RightParentheses);
    let return_type = parse_type(tokens);
    let code_block = get_block_commands(tokens);
    Identifier::Function(Function {
        name,
        parameters,
        return_type,
        code_block,
    })
}

fn get_init_declarations(tokens: &mut VecDeque<Token>) -> Vec<Identifier> {
    let mut declarations: Vec<Identifier> = Vec::new();
    loop {
        skip_whitespace(tokens);

        match tokens.front() {
            Some(token) if token.class == TokenClass::KeyWord => {
                match KeyWord::from_lexema(token.get_lexema()) {
                    Some(KeyWord::Func) => {
                        declarations.push(extract_function_declaration(tokens))
                    }
                    Some(k) if Type::from_keyword(k).is_some() => {
                        declarations.push(extract_variable_declaration(tokens))
                    }
                    _ => handle_sintax_error(
                        "only typed variable or function declarations are available in global scope",
                    ),
                }
            }
            _ => return declarations,
        }
    }
}

fn get_attribution(tokens: &mut VecDeque<Token>) -> Command {
    let Some(variable) = tokens.pop_front() else {
        handle_sintax_error("Attribution without variable")
    };
    let target = match next_token_class(tokens, 0) {
        Some(TokenClass::LeftBracket) => {
            consume_token_class(tokens, TokenClass::LeftBracket);
            let index = extract_expression(tokens);
            consume_token_class(tokens, TokenClass::RightBracket);
            AssignTarget::Index {
                array: variable,
                index,
            }
        }
        _ => AssignTarget::Variable(variable),
    };
    consume_token_class(tokens, TokenClass::Attribution);
    let expression = extract_expression(tokens);
    consume_token_class(tokens, TokenClass::Semicolon);
    Command::Attribution { target, expression }
}

fn get_compound_attribution(tokens: &mut VecDeque<Token>, operator: Operator) -> Command {
    let Some(variable) = tokens.pop_front() else {
        handle_sintax_error("Compound attribution without variable")
    };
    let variable_name = variable.get_lexema().to_string();
    let expected = match operator {
        Operator::Sum => TokenClass::PlusEqual,
        Operator::Sub => TokenClass::MinusEqual,
        Operator::Mul => TokenClass::MulEqual,
        Operator::Div => TokenClass::DivEqual,
        _ => unreachable!("invalid compound attribution operator"),
    };
    consume_token_class(tokens, expected);
    let right = extract_expression(tokens);
    consume_token_class(tokens, TokenClass::Semicolon);
    Command::Attribution {
        target: AssignTarget::Variable(variable),
        expression: Expression::BinOperation {
            left_value: Box::new(Expression::Identifier(variable_name)),
            operator,
            right_value: Box::new(right),
        },
    }
}

fn get_increment(tokens: &mut VecDeque<Token>) -> Command {
    let Some(variable) = tokens.pop_front() else {
        handle_sintax_error("Increment without variable")
    };
    let variable_name = variable.get_lexema().to_string();
    consume_token_class(tokens, TokenClass::Increment);
    consume_token_class(tokens, TokenClass::Semicolon);
    Command::Attribution {
        target: AssignTarget::Variable(variable),
        expression: Expression::BinOperation {
            left_value: Box::new(Expression::Identifier(variable_name)),
            operator: Operator::Sum,
            right_value: Box::new(Expression::NumberLiteral(1)),
        },
    }
}

fn read_call_parameters(tokens: &mut VecDeque<Token>, parameters: &mut Vec<Expression>) {
    skip_whitespace(tokens);

    match tokens.front() {
        Some(token) => match token.class {
            TokenClass::RightParentheses => {
                return consume_token_class(tokens, TokenClass::RightParentheses)
            }
            TokenClass::Comma => {
                consume_token_class(tokens, TokenClass::Comma);
                let expression = extract_expression(tokens);
                parameters.push(expression);
                read_call_parameters(tokens, parameters);
            }
            _ => {
                let expression = extract_expression(tokens);
                parameters.push(expression);
                read_call_parameters(tokens, parameters);
            }
        },
        None => handle_sintax_error("function call incomplete"),
    }
}

fn get_function_call(tokens: &mut VecDeque<Token>) -> Command {
    let expr = extract_expression(tokens);
    skip_whitespace(tokens);
    if next_token_class(tokens, 0) == Some(TokenClass::Semicolon) {
        consume_token_class(tokens, TokenClass::Semicolon);
    }
    match expr {
        Expression::FunctionCall { name, parameters } => Command::FunctionCall { name, parameters },
        _ => handle_sintax_error("statement expression must be a function/method call"),
    }
}

fn get_block_commands(tokens: &mut VecDeque<Token>) -> CodeBlock {
    consume_token_class(tokens, TokenClass::OpenBlock);
    let mut commands: Vec<Command> = Vec::new();
    loop {
        skip_whitespace(tokens);
        let class = match tokens.front() {
            Some(token) => token.class,
            None => {
                handle_sintax_error("block of comands without end close");
            }
        };

        commands.push(match class {
            TokenClass::CloseBlock => break,
            TokenClass::Identifier => match next_token_class(tokens, 1) {
                Some(TokenClass::Attribution) => get_attribution(tokens),
                Some(TokenClass::LeftBracket) => get_attribution(tokens),
                Some(TokenClass::PlusEqual) => get_compound_attribution(tokens, Operator::Sum),
                Some(TokenClass::MinusEqual) => get_compound_attribution(tokens, Operator::Sub),
                Some(TokenClass::MulEqual) => get_compound_attribution(tokens, Operator::Mul),
                Some(TokenClass::DivEqual) => get_compound_attribution(tokens, Operator::Div),
                Some(TokenClass::Increment) => get_increment(tokens),
                Some(TokenClass::LeftParentheses) | Some(TokenClass::Dot) => {
                    get_function_call(tokens)
                }
                _ => handle_sintax_error("wrong use of identifier"),
            },
            TokenClass::KeyWord => {
                let keyword = {
                    let Some(token) = tokens.front() else {
                        handle_sintax_error(";-;")
                    };
                    KeyWord::from_lexema(token.get_lexema())
                };

                match keyword {
                    Some(keyword) => match keyword {
                        KeyWord::If => {
                            consume_token_class(tokens, TokenClass::KeyWord);
                            let condition = extract_expression(tokens);
                            let true_block = get_block_commands(tokens);

                            skip_whitespace(tokens);

                            let has_else = tokens
                                .front()
                                .map(|t| is_keyword(t, KeyWord::Else))
                                .unwrap_or(false);
                            if has_else {
                                consume_token_class(tokens, TokenClass::KeyWord);
                                let false_block = get_block_commands(tokens);
                                Command::If {
                                    condition,
                                    true_block,
                                    false_block,
                                }
                            } else {
                                handle_sintax_error("condition must have else block");
                            }
                        }
                        KeyWord::While => {
                            consume_token_class(tokens, TokenClass::KeyWord);
                            let condition = extract_expression(tokens);
                            let block = get_block_commands(tokens);
                            Command::While { condition, block }
                        }
                        KeyWord::Return => {
                            consume_token_class(tokens, TokenClass::KeyWord);
                            let expression = extract_expression(tokens);
                            consume_token_class(tokens, TokenClass::Semicolon);
                            Command::Return { expression }
                        }
                        KeyWord::Print => {
                            consume_token_class(tokens, TokenClass::KeyWord);
                            let expression = extract_expression(tokens);
                            consume_token_class(tokens, TokenClass::Semicolon);
                            Command::Print { expression }
                        }
                        KeyWord::Func => Command::Declaration {
                            identifier: extract_function_declaration(tokens),
                        },
                        k if Type::from_keyword(k).is_some() => Command::Declaration {
                            identifier: extract_variable_declaration(tokens),
                        },
                        _ => handle_sintax_error(&format!(
                            "{:?} cannot be used as a command",
                            keyword
                        )),
                    },
                    None => handle_sintax_error("keyword unexpected"),
                }
            }
            _ => handle_sintax_error("command unexpected"),
        });
    }
    consume_token_class(tokens, TokenClass::CloseBlock);
    CodeBlock { commands }
}

pub fn build_program(tokens_list: TokenList) -> Program {
    let mut tokens = VecDeque::from(tokens_list.get_tokens());
    let declarations = get_init_declarations(&mut tokens);
    Program { declarations }
}
