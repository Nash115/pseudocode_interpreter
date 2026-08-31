use crate::frontend::errors::{
    LexerError::{self, *},
    LexerErrorWithSpan,
};
use crate::frontend::span::{Position, Span};

use self::TokenType::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Litral
    Number,
    StringToken,
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
    For,
    In,
    ForEnd,
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
    pub span: Span,
}
impl Token {
    pub fn new(value: String, token_type: TokenType, line: usize, col: usize) -> Token {
        Token {
            value,
            token_type: token_type,
            span: Span {
                start: Position { line, col },
                end: Position { line, col },
            },
        }
    }
    pub fn new_multichar(
        value: String,
        token_type: TokenType,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Token {
        Token {
            value,
            token_type: token_type,
            span: Span {
                start: Position {
                    line: start_line,
                    col: start_col,
                },
                end: Position {
                    line: end_line,
                    col: end_col,
                },
            },
        }
    }
}

fn keyword(str: String) -> TokenType {
    match str.as_str() {
        // Let
        "var" => Let,
        "let" => Let,
        "variable" => Let,
        // Const
        "const" => Const,
        "constante" => Const,
        // If
        "if" => If,
        "si" => If,
        "then" => Then,
        "alors" => Then,
        "else" => Else,
        "sinon" => Else,
        "endIf" => IfEnd,
        "finSi" => IfEnd,
        // Loops
        "while" => While,
        "tantQue" => While,
        "endWhile" => WhileEnd,
        "finTantQue" => WhileEnd,
        "for" => For,
        "pour" => For,
        "in" => In,
        "dans" => In,
        "endFor" => ForEnd,
        "finPour" => ForEnd,
        // Fn
        "fn" => FnStart,
        "function" => FnStart,
        "fonction" => FnStart,
        "procedure" => FnStart,
        "endFn" => FnEnd,
        "finFn" => FnEnd,
        "endFunction" => FnEnd,
        "finFonction" => FnEnd,
        "endProcedure" => FnEnd,
        "finProcedure" => FnEnd,
        "return" => Return,
        "retourner" => Return,
        "renvoyer" => Return,
        // Logical operators
        "not" => UnaryOperator,
        "non" => UnaryOperator,
        "&&" => LogicalOperator,
        "and" => LogicalOperator,
        "et" => LogicalOperator,
        "||" => LogicalOperator,
        "or" => LogicalOperator,
        "ou" => LogicalOperator,
        "is" => LogicalOperator,
        "est" => LogicalOperator,
        // Fallback
        _ => Identifier,
    }
}

fn skippable(car: char) -> bool {
    car.is_whitespace()
}

pub fn tokenize(source_code: &str) -> Result<Vec<Token>, LexerErrorWithSpan> {
    let mut tokens = Vec::new();
    let mut chars = source_code.chars().peekable();

    let mut line: usize = 1;
    let mut col: usize = 1;

    while let Some(&c) = chars.peek() {
        if c == '\n' {
            line += 1;
            col = 1;
            chars.next();
        } else if c == '(' {
            tokens.push(Token::new(c.to_string(), OpenParen, line, col));
            chars.next();
            col += 1;
        } else if c == ')' {
            tokens.push(Token::new(c.to_string(), CloseParen, line, col));
            chars.next();
            col += 1;
        } else if c == '{' {
            tokens.push(Token::new(c.to_string(), OpenBrace, line, col));
            chars.next();
            col += 1;
        } else if c == '}' {
            tokens.push(Token::new(c.to_string(), CloseBrace, line, col));
            chars.next();
            col += 1;
        } else if c == '[' {
            tokens.push(Token::new(c.to_string(), OpenBracket, line, col));
            chars.next();
            col += 1;
        } else if c == ']' {
            tokens.push(Token::new(c.to_string(), CloseBracket, line, col));
            chars.next();
            col += 1;
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
            col += 1;
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
                tokens.push(Token::new(c.to_string(), BinaryOperator, line, col));
            }
        } else if c == '+' || c == '-' || c == '*' || c == '%' {
            tokens.push(Token::new(c.to_string(), BinaryOperator, line, col));
            chars.next();
            col += 1;
        } else if c == '!' {
            chars.next();
            col += 1;
            if let Some(&next_c) = chars.peek()
                && next_c == '='
            {
                tokens.push(Token::new("!=".to_string(), LogicalOperator, line, col));
                chars.next();
                col += 1;
            } else {
                tokens.push(Token::new(c.to_string(), UnaryOperator, line, col));
            }
        } else if c == '<' || c == '>' {
            chars.next();
            col += 1;
            if let Some(&next_c) = chars.peek()
                && next_c == '='
            {
                tokens.push(Token::new(format!("{}=", c), LogicalOperator, line, col));
                chars.next();
                col += 1;
            } else {
                tokens.push(Token::new(c.to_string(), LogicalOperator, line, col));
            }
        } else if c == '"' || c == '\'' {
            chars.next();
            col += 1;
            let mut s = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == c {
                    break;
                } else {
                    s.push(next_c);
                }
                chars.next();
                col += 1;
            }
            tokens.push(Token::new(s, StringToken, line, col));
            chars.next();
            col += 1;
        } else if c == '=' {
            chars.next();
            col += 1;
            if let Some(&next_c) = chars.peek()
                && next_c == '='
            {
                tokens.push(Token::new("==".to_string(), LogicalOperator, line, col));
                chars.next();
                col += 1;
            } else {
                tokens.push(Token::new(c.to_string(), Equals, line, col));
            }
        } else if c == ',' {
            tokens.push(Token::new(c.to_string(), Comma, line, col));
            chars.next();
            col += 1;
        } else if c == '.' {
            tokens.push(Token::new(c.to_string(), Dot, line, col));
            chars.next();
            col += 1;
        } else if c == ':' {
            tokens.push(Token::new(c.to_string(), Colon, line, col));
            chars.next();
            col += 1;
        } else if c == ';' {
            tokens.push(Token::new(c.to_string(), Semicolon, line, col));
            chars.next();
            col += 1;
        } else {
            // Multicharacter tokens
            let start_line = line;
            let start_col = col;
            if c.is_alphabetic() || c == '_' {
                let mut ident = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        ident.push(next_c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                let t: TokenType = keyword(ident.clone());
                match t {
                    UnaryOperator => tokens.push(Token::new("!".to_string(), t, line, col)),
                    LogicalOperator => tokens.push(Token::new_multichar(
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
                        start_line,
                        start_col,
                        line,
                        col,
                    )),
                    _ => tokens.push(Token::new_multichar(
                        ident, t, start_line, start_col, line, col,
                    )),
                }
            } else if c.is_numeric() {
                let mut num = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_numeric() || next_c == '.' {
                        num.push(next_c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token::new_multichar(
                    num, Number, start_line, start_col, line, col,
                ));
            } else if skippable(c) {
                chars.next();
                col += 1;
            } else {
                return Err(LexerError::with_span(
                    UnknownCharacter(c),
                    Span {
                        start: Position { line, col },
                        end: Position { line, col },
                    },
                ));
            }
        }
    }

    tokens.push(Token::new("EOF".to_string(), EOF, line, col));

    Ok(tokens)
}
