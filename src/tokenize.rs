use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    class: TokenClass,
    lexema: String,
    column: usize,
    line: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenLength {
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenClass {
    Number,
    LeftParentheses,
    RightParentheses,
    SumOperator,
    SubOperator,
    DivOperator,
    MulOperator,
    Space,
    NewLine,
}

impl TokenClass {
    fn from_char(c: char) -> Option<TokenClass> {
        match c {
            '0'..='9' => Some(TokenClass::Number),
            '(' => Some(TokenClass::LeftParentheses),
            ')' => Some(TokenClass::RightParentheses),
            '+' => Some(TokenClass::SumOperator),
            '-' => Some(TokenClass::SubOperator),
            '/' => Some(TokenClass::DivOperator),
            '*' => Some(TokenClass::MulOperator),
            '\n' => Some(TokenClass::NewLine),
            ' ' | '\t' => Some(TokenClass::Space),
            _ => None,
        }
    }

    fn lenght_type(self) -> TokenLength {
        match self {
            TokenClass::Number => TokenLength::Multi,
            TokenClass::Space => TokenLength::Multi,
            TokenClass::NewLine => TokenLength::Multi,
            _ => TokenLength::Single,
        }
    }
}

pub struct TokenList {
    tokens: Vec<Token>,
}

impl TokenList {
    fn push_token(&mut self, class: TokenClass, c: char, column: usize, line: usize) {
        self.tokens.push(Token {
            class,
            lexema: String::from(c),
            column,
            line,
        });
    }

    fn complement_last_token(&mut self, c: char) {
        if let Some(last) = self.tokens.last_mut() {
            last.lexema.push(c);
        }
    }

    fn push_char(&mut self, c: char, column: usize, line: usize) {
        if let Some(token_class) = TokenClass::from_char(c) {
            match token_class.lenght_type() {
                TokenLength::Multi => {
                    if let Some(last) = self.tokens.last()
                        && last.class == token_class
                    {
                        self.complement_last_token(c);
                    } else {
                        self.push_token(token_class, c, column, line);
                    }
                }
                TokenLength::Single => {
                    self.push_token(token_class, c, column, line);
                }
            }
        } else {
            handle_invalid_token(c, column, line)
        }
    }
}

impl fmt::Display for TokenList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in &self.tokens {
            writeln!(
                f,
                "Token::{:?} ({}) at {}:{}",
                t.class, t.lexema, t.line, t.column
            )?;
        }
        Ok(())
    }
}

fn handle_invalid_token(c: char, column: usize, line: usize) {
    eprintln!("Error: invalid character '{}' at {}:{}", c, line, column);
    std::process::exit(1);
}

pub fn extract_tokens(text: &String) -> TokenList {
    let mut tokens: TokenList = TokenList { tokens: Vec::new() };
    let mut current_line = 1;
    let mut current_column: usize = 1;

    for (index, c) in text.char_indices() {
        if c == '\n' {
            current_line += 1;
            current_column = 0;
        }
        tokens.push_char(c, current_column, current_line);
        current_column += 1
    }

    return tokens;
}
