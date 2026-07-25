use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub class: TokenClass,
    lexema: String,
    column: usize,
    line: usize,
}

impl Token {
    pub fn new(class: TokenClass, lexema: String, column: usize, line: usize) -> Self {
        Token {
            class,
            lexema,
            column,
            line,
        }
    }

    pub fn get_number(self) -> Option<i32> {
        match self.class {
            TokenClass::Number => Some(
                self.lexema
                    .parse::<i32>()
                    .expect("Number token expected to parse into i32"),
            ),
            _ => None,
        }
    }

    pub fn get_lexema(&self) -> &String {
        &self.lexema
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenClass {
    Number,
    Attribution,
    Semicolon,
    Identifier,

    LeftParentheses,
    RightParentheses,

    OpenBlock,
    CloseBlock,

    SumOperator,
    SubOperator,
    DivOperator,
    MulOperator,
    ModOperator,

    EqualOperator,
    LessThanOperator,
    LessEqualOperator,
    GreaterThanOperator,
    GreaterEqualOperator,
    AndOperator,
    NotOperator,
    OrOperator,

    Space,
    NewLine,

    KeyWord,
}

#[derive(PartialEq, Eq)]
pub enum KeyWord {
    If,
    While,
    Else,
    Return,
    Print,
}

impl KeyWord {
    pub fn from_lexema(lexema: &String) -> Option<KeyWord> {
        match lexema.as_str() {
            "if" => Some(KeyWord::If),
            "else" => Some(KeyWord::Else),
            "while" => Some(KeyWord::While),
            "return" => Some(KeyWord::Return),
            "print" => Some(KeyWord::Print),
            _ => None,
        }
    }
}

pub struct TokenList {
    tokens: Vec<Token>,
}

impl TokenList {
    pub fn get_tokens(self) -> Vec<Token> {
        self.tokens
    }

    fn push(&mut self, token: Token) {
        self.tokens.push(token);
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

fn handle_invalid_token(c: char, column: usize, line: usize) -> ! {
    eprintln!(
        "Error lexical: invalid character '{}' at {}:{}",
        c, line, column
    );
    std::process::exit(1);
}

fn is_letter(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z')
}

fn is_digit(c: char) -> bool {
    matches!(c, '0'..='9')
}

fn is_letter_or_digit(c: char) -> bool {
    is_letter(c) || is_digit(c)
}

fn read_identifier(chars: &[char], read_index: usize) -> usize {
    let mut literal_index = read_index + 1;
    while literal_index < chars.len() && is_letter_or_digit(chars[literal_index]) {
        literal_index += 1;
    }
    literal_index
}

fn read_number(chars: &[char], read_index: usize) -> usize {
    let mut literal_index = read_index + 1;
    while literal_index < chars.len() && is_digit(chars[literal_index]) {
        literal_index += 1;
    }
    literal_index
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

fn read_spaces(chars: &[char], read_index: usize) -> usize {
    let mut literal_index = read_index + 1;
    while literal_index < chars.len() && is_space(chars[literal_index]) {
        literal_index += 1;
    }
    literal_index
}

fn read_token(chars: &[char], read_index: usize, column: usize, line: usize) -> (Token, usize) {
    let c = chars[read_index];

    match c {
        'a'..='z' | 'A'..='Z' => {
            let end = read_identifier(chars, read_index);
            let lexema: String = chars[read_index..end].iter().collect();
            let class = match lexema.as_str() {
                "and" => TokenClass::AndOperator,
                "not" => TokenClass::NotOperator,
                "or" => TokenClass::OrOperator,
                _ if KeyWord::from_lexema(&lexema).is_some() => TokenClass::KeyWord,
                _ => TokenClass::Identifier,
            };
            (Token::new(class, lexema, column, line), end)
        }
        '0'..='9' => {
            let end = read_number(chars, read_index);
            let lexema: String = chars[read_index..end].iter().collect();
            (Token::new(TokenClass::Number, lexema, column, line), end)
        }
        '=' => {
            if read_index + 1 < chars.len() && chars[read_index + 1] == '=' {
                (
                    Token::new(TokenClass::EqualOperator, String::from("=="), column, line),
                    read_index + 2,
                )
            } else {
                (
                    Token::new(TokenClass::Attribution, String::from("="), column, line),
                    read_index + 1,
                )
            }
        }
        '<' => {
            if read_index + 1 < chars.len() && chars[read_index + 1] == '=' {
                (
                    Token::new(
                        TokenClass::LessEqualOperator,
                        String::from("<="),
                        column,
                        line,
                    ),
                    read_index + 2,
                )
            } else {
                (
                    Token::new(
                        TokenClass::LessThanOperator,
                        String::from("<"),
                        column,
                        line,
                    ),
                    read_index + 1,
                )
            }
        }
        '>' => {
            if read_index + 1 < chars.len() && chars[read_index + 1] == '=' {
                (
                    Token::new(
                        TokenClass::GreaterEqualOperator,
                        String::from(">="),
                        column,
                        line,
                    ),
                    read_index + 2,
                )
            } else {
                (
                    Token::new(
                        TokenClass::GreaterThanOperator,
                        String::from(">"),
                        column,
                        line,
                    ),
                    read_index + 1,
                )
            }
        }
        '{' => (
            Token::new(TokenClass::OpenBlock, String::from("{"), column, line),
            read_index + 1,
        ),
        '}' => (
            Token::new(TokenClass::CloseBlock, String::from("}"), column, line),
            read_index + 1,
        ),
        ';' => (
            Token::new(TokenClass::Semicolon, String::from(";"), column, line),
            read_index + 1,
        ),
        '(' => (
            Token::new(TokenClass::LeftParentheses, String::from("("), column, line),
            read_index + 1,
        ),
        ')' => (
            Token::new(
                TokenClass::RightParentheses,
                String::from(")"),
                column,
                line,
            ),
            read_index + 1,
        ),
        '+' => (
            Token::new(TokenClass::SumOperator, String::from("+"), column, line),
            read_index + 1,
        ),
        '-' => (
            Token::new(TokenClass::SubOperator, String::from("-"), column, line),
            read_index + 1,
        ),
        '/' => (
            Token::new(TokenClass::DivOperator, String::from("/"), column, line),
            read_index + 1,
        ),
        '*' => (
            Token::new(TokenClass::MulOperator, String::from("*"), column, line),
            read_index + 1,
        ),
        '%' => (
            Token::new(TokenClass::ModOperator, String::from("%"), column, line),
            read_index + 1,
        ),
        ' ' | '\t' => {
            let end = read_spaces(chars, read_index);
            let lexema: String = chars[read_index..end].iter().collect();
            (Token::new(TokenClass::Space, lexema, column, line), end)
        }
        '\n' | '\r' => (
            Token::new(
                TokenClass::NewLine,
                String::from(chars[read_index]),
                column,
                line,
            ),
            read_index + 1,
        ),
        _ => handle_invalid_token(c, column, line),
    }
}

pub fn extract_tokens(text: &String) -> TokenList {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = TokenList { tokens: Vec::new() };
    let mut read_index = 0;
    let mut current_line = 1;
    let mut current_column: usize = 1;

    while read_index < chars.len() {
        // let c = chars[read_index];
        let (token, next_index) = read_token(&chars, read_index, current_column, current_line);
        if token.class == TokenClass::NewLine {
            current_line += 1;
        }
        current_column += next_index - read_index;
        read_index = next_index;
        tokens.push(token);
    }

    tokens
}
