use crate::frontend::errors::LexerError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Litral
    Number,
    String,
    Identifier,

    // Keywords
    Let,
    Const,
    If,
    Then,
    Else,
    IfEnd,
    While,
    WhileEnd,
    FnStart,
    FnEnd,
    Return,

    // Grouping, Operators
    BinaryOperator,
    UnaryOperator,
    LogicalOperator,
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
        // Let
        "var" => TokenType::Let,
        "let" => TokenType::Let,
        "variable" => TokenType::Let,
        // Const
        "const" => TokenType::Const,
        "constante" => TokenType::Const,
        // If
        "if" => TokenType::If,
        "si" => TokenType::If,
        "then" => TokenType::Then,
        "alors" => TokenType::Then,
        "else" => TokenType::Else,
        "sinon" => TokenType::Else,
        "endIf" => TokenType::IfEnd,
        "finSi" => TokenType::IfEnd,
        // Loops
        "while" => TokenType::While,
        "tantQue" => TokenType::While,
        "endWhile" => TokenType::WhileEnd,
        "finTantQue" => TokenType::WhileEnd,
        // Fn
        "fn" => TokenType::FnStart,
        "function" => TokenType::FnStart,
        "fonction" => TokenType::FnStart,
        "procedure" => TokenType::FnStart,
        "endFn" => TokenType::FnEnd,
        "finFn" => TokenType::FnEnd,
        "endFunction" => TokenType::FnEnd,
        "finFonction" => TokenType::FnEnd,
        "endProcedure" => TokenType::FnEnd,
        "finProcedure" => TokenType::FnEnd,
        "return" => TokenType::Return,
        "retourner" => TokenType::Return,
        "renvoyer" => TokenType::Return,
        // Logical operators
        "not" => TokenType::UnaryOperator,
        "non" => TokenType::UnaryOperator,
        "&&" => TokenType::LogicalOperator,
        "and" => TokenType::LogicalOperator,
        "et" => TokenType::LogicalOperator,
        "||" => TokenType::LogicalOperator,
        "or" => TokenType::LogicalOperator,
        "ou" => TokenType::LogicalOperator,
        "is" => TokenType::LogicalOperator,
        "est" => TokenType::LogicalOperator,
        // Fallback
        _ => TokenType::Identifier,
    }
}

fn skippable(car: char) -> bool {
    car.is_whitespace()
}

pub fn tokenize(source_code: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut chars = source_code.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '(' {
            tokens.push(Token::new(c.to_string(), TokenType::OpenParen));
            chars.next();
        } else if c == ')' {
            tokens.push(Token::new(c.to_string(), TokenType::CloseParen));
            chars.next();
        } else if c == '{' {
            tokens.push(Token::new(c.to_string(), TokenType::OpenBrace));
            chars.next();
        } else if c == '}' {
            tokens.push(Token::new(c.to_string(), TokenType::CloseBrace));
            chars.next();
        } else if c == '[' {
            tokens.push(Token::new(c.to_string(), TokenType::OpenBracket));
            chars.next();
        } else if c == ']' {
            tokens.push(Token::new(c.to_string(), TokenType::CloseBracket));
            chars.next();
        } else if c == '#' {
            while let Some(&next_c) = chars.peek() {
                if next_c == '\n' {
                    break;
                }
                chars.next();
            }
        } else if c == '/' {
            // Handle Division and '//' Comments
            chars.next();
            if let Some(&'/') = chars.peek() {
                // COMMENT
                chars.next();
                while let Some(&next_c) = chars.peek() {
                    if next_c == '\n' {
                        break;
                    }
                    chars.next();
                }
            } else {
                // DIVISION
                tokens.push(Token::new(c.to_string(), TokenType::BinaryOperator));
            }
        } else if c == '+' || c == '-' || c == '*' || c == '%' {
            tokens.push(Token::new(c.to_string(), TokenType::BinaryOperator));
            chars.next();
        } else if c == '!' {
            chars.next();
            if let Some(&next_c) = chars.peek()
                && next_c == '='
            {
                tokens.push(Token::new("!=".to_string(), TokenType::LogicalOperator));
                chars.next();
            } else {
                tokens.push(Token::new(c.to_string(), TokenType::UnaryOperator));
            }
        } else if c == '<' || c == '>' {
            chars.next();
            if let Some(&next_c) = chars.peek()
                && next_c == '='
            {
                tokens.push(Token::new(format!("{}=", c), TokenType::LogicalOperator));
                chars.next();
            } else {
                tokens.push(Token::new(c.to_string(), TokenType::LogicalOperator));
            }
        } else if c == '"' || c == '\'' {
            chars.next();
            let mut s = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == c {
                    break;
                } else {
                    s.push(next_c);
                }
                chars.next();
            }
            tokens.push(Token::new(s, TokenType::String));
            chars.next();
        } else if c == '=' {
            chars.next();
            if let Some(&next_c) = chars.peek()
                && next_c == '='
            {
                tokens.push(Token::new("==".to_string(), TokenType::LogicalOperator));
                chars.next();
            } else {
                tokens.push(Token::new(c.to_string(), TokenType::Equals));
            }
        } else if c == ',' {
            tokens.push(Token::new(c.to_string(), TokenType::Comma));
            chars.next();
        } else if c == '.' {
            tokens.push(Token::new(c.to_string(), TokenType::Dot));
            chars.next();
        } else if c == ':' {
            tokens.push(Token::new(c.to_string(), TokenType::Colon));
            chars.next();
        } else if c == ';' {
            tokens.push(Token::new(c.to_string(), TokenType::Semicolon));
            chars.next();
        } else {
            // Multicharacter tokens
            if c.is_alphabetic() {
                let mut ident = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() {
                        ident.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let t: TokenType = keyword(ident.clone());
                match t {
                    TokenType::UnaryOperator => tokens.push(Token::new("!".to_string(), t)),
                    TokenType::LogicalOperator => tokens.push(Token::new(
                        match ident.as_str() {
                            "and" => "&&".to_string(),
                            "et" => "&&".to_string(),
                            "or" => "||".to_string(),
                            "ou" => "||".to_string(),
                            "is" => "==".to_string(),
                            "est" => "==".to_string(),
                            _ => ident,
                        },
                        t,
                    )),
                    _ => tokens.push(Token::new(ident, t)),
                }
            } else if c.is_numeric() {
                let mut num = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_numeric() || next_c == '.' {
                        num.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::new(num, TokenType::Number));
            } else if skippable(c) {
                chars.next();
            } else {
                return Err(LexerError::UnknownCharacter(c));
            }
        }
    }

    tokens.push(Token::new("EOF".to_string(), TokenType::EOF));

    Ok(tokens)
}
