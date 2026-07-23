pub enum LexerErrors {
    UnknownCharacter(char),
}

impl std::fmt::Display for LexerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrors::UnknownCharacter(c) => write!(f, "Unknown character '{}'", c),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Litral
    Number,
    Identifier,

    // Keywords
    Let,
    Const,

    // Grouping, Operators
    BinaryOperator,
    Equals,
    Comma,
    Dot,
    Colon,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,

    // End Of File
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub token_type: TokenType,
}
impl Token {
    pub fn new(value: String, token_type: TokenType) -> Token {
        Token {
            value,
            token_type: token_type,
        }
    }
}

fn keyword(str: String) -> TokenType {
    match str.as_str() {
        "VARIABLE" => TokenType::Let,
        "CONSTANTE" => TokenType::Const,
        _ => TokenType::Identifier,
    }
}

fn skippable(car: char) -> bool {
    return if car == ' ' || car == '\n' || car == '\t' || car == '\r' {
        true
    } else {
        false
    };
}

pub fn tokenize(source_code: &str) -> Result<Vec<Token>, LexerErrors> {
    let mut tokens = Vec::new();
    let mut src = source_code.chars().collect::<Vec<char>>();

    while src.len() > 0 {
        if src[0] == '(' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::OpenParen));
        } else if src[0] == ')' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::CloseParen));
        } else if src[0] == '{' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::OpenBrace));
        } else if src[0] == '}' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::CloseBrace));
        } else if src[0] == '[' {
            tokens.push(Token::new(
                src.remove(0).to_string(),
                TokenType::OpenBracket,
            ));
        } else if src[0] == ']' {
            tokens.push(Token::new(
                src.remove(0).to_string(),
                TokenType::CloseBracket,
            ));
        } else if src[0] == '+' || src[0] == '-' || src[0] == '*' || src[0] == '/' || src[0] == '%'
        {
            tokens.push(Token::new(
                src.remove(0).to_string(),
                TokenType::BinaryOperator,
            ));
        } else if src[0] == '=' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::Equals));
        } else if src[0] == ',' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::Comma));
        } else if src[0] == '.' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::Dot));
        } else if src[0] == ':' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::Colon));
        } else if src[0] == ';' {
            tokens.push(Token::new(src.remove(0).to_string(), TokenType::Semicolon));
        } else {
            // Multicharacter tokens
            if src[0].is_alphabetic() {
                let mut ident: String = "".to_string();
                while src.len() > 0 && src[0].is_alphanumeric() {
                    ident = format!("{}{}", ident, src.remove(0));
                }
                let t: TokenType = keyword(ident.clone());
                tokens.push(Token::new(ident, t));
            } else if src[0].is_numeric() {
                let mut num: String = "".to_string();
                while src.len() > 0 && src[0].is_numeric() {
                    num = format!("{}{}", num, src.remove(0));
                }
                tokens.push(Token::new(num, TokenType::Number));
            } else if skippable(src[0]) {
                src.remove(0);
            } else {
                return Err(LexerErrors::UnknownCharacter(src[0]));
            }
        }
    }

    tokens.push(Token::new("EOF".to_string(), TokenType::EOF));

    Ok(tokens)
}
