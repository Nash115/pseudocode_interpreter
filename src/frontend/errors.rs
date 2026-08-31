use std::num::ParseFloatError;

use crate::frontend::ast::Expr;
use crate::frontend::lexer::{Token, TokenType};
use crate::frontend::span::Span;
use crate::runtime::values::RuntimeVal;

pub struct Colors;
#[allow(dead_code)]
impl Colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";

    pub fn error(message: &str) -> String {
        format!("{}{}{}", Colors::RED, message, Colors::RESET)
    }
    pub fn hint(message: &str) -> String {
        format!("{}{}{}", Colors::BLUE, message, Colors::RESET)
    }
    pub fn warn(message: &str) -> String {
        format!("{}{}{}", Colors::YELLOW, message, Colors::RESET)
    }
}

pub fn format_error_snippet(source_code: &str, span: &Span) -> String {
    if span.is_null() {
        return String::from("(Unknown location)");
    }
    let line_index = span.start.line.saturating_sub(1);
    let line_text = source_code.lines().nth(line_index).unwrap_or("");

    let col = span.start.col;
    let pointer_padding = " ".repeat(col.saturating_sub(1));

    let token_len = if span.start.line == span.end.line {
        span.end.col.saturating_sub(span.start.col).max(1)
    } else {
        line_text.len().saturating_sub(col.saturating_sub(1)).max(1)
    };

    let pointers = "^".repeat(token_len);

    format!(
        "  --> {}\n{:>3} | {}\n    | {}{}",
        span,
        span.start.line,
        line_text,
        pointer_padding,
        Colors::warn(&pointers)
    )
}

#[derive(Debug)]
pub struct LexerErrorWithSpan {
    pub error: LexerError,
    pub span: Span,
}
impl std::error::Error for LexerErrorWithSpan {}
impl std::fmt::Display for LexerErrorWithSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.error);
    }
}
#[derive(Debug)]
pub enum LexerError {
    UnknownCharacter(char),
}
impl std::error::Error for LexerError {}
impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = Colors::error("[LEXER ERROR]");
        match self {
            LexerError::UnknownCharacter(c) => {
                return write!(f, "{} Unexpected character '{}'.", prefix, c);
            }
        }
    }
}
impl LexerError {
    pub fn with_span(self, span: Span) -> LexerErrorWithSpan {
        LexerErrorWithSpan { error: self, span }
    }
}

#[derive(Debug)]
pub struct ParserErrorWithSpan {
    pub error: ParserError,
    pub span: Span,
}
impl std::error::Error for ParserErrorWithSpan {}
impl std::fmt::Display for ParserErrorWithSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.error);
    }
}
#[derive(Debug)]
pub enum ParserError {
    TokenExpected {
        expected: TokenType,
        found: Token,
        hint: String,
    },
    Unexpected {
        expected: String,
        hint: String,
    },
    UnexpectedToken {
        found: Token,
        hint: String,
    },
    ParsingNumber(ParseFloatError),
}
impl std::error::Error for ParserError {}
impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = Colors::error("[PARSER ERROR]");
        match self {
            ParserError::TokenExpected {
                expected,
                found,
                hint,
            } => {
                return write!(
                    f,
                    "{} Expecting {:?}, found {:?} : {}.\n  {} : {}",
                    prefix,
                    expected,
                    found.token_type,
                    found.value,
                    Colors::hint("HINT"),
                    hint
                );
            }
            ParserError::Unexpected { expected, hint } => {
                return write!(
                    f,
                    "{} Expecting {}.\n  {} : {}",
                    prefix,
                    expected,
                    Colors::hint("HINT"),
                    hint
                );
            }
            ParserError::UnexpectedToken { found, hint } => {
                return write!(
                    f,
                    "{} Unexpected token {:?} : '{}'.\n  {} : {}",
                    prefix,
                    found.token_type,
                    found.value,
                    Colors::hint("HINT"),
                    hint
                );
            }
            ParserError::ParsingNumber(e) => {
                return write!(f, "{} Error parsing float.\n  {}", prefix, e);
            }
        }
    }
}
impl ParserError {
    pub fn with_span(self, span: Span) -> ParserErrorWithSpan {
        ParserErrorWithSpan { error: self, span }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub error: InterpreterError,
    pub span: Span,
}
impl std::error::Error for RuntimeError {}
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.error);
    }
}
#[derive(Debug)]
pub enum InterpreterError {
    UnexpectedReturn,
    DivBy0,
    VarAlreadyDeclared(String),
    VarUnresolvable(String),
    EditConst(String),
    UnknownBinaryOperator(String),
    UnpermittedBinaryOperation {
        lhs: String,
        rhs: String,
        operator: String,
    },
    UnknownUnaryOperator(String),
    UnknownLogicalOperator(String),
    NumberInterpretation(RuntimeVal),
    Assignment(Expr),
    ObjectKeyUncomputedNotIdentifier(Expr),
    ObjectKeyComputedType(RuntimeVal),
    MemberNotAccessible {
        action: String,
        value: RuntimeVal,
    },
    InvalidIndex(String),
    OutOfBounds(i64),
    NotAFunction {
        action: String,
        value: RuntimeVal,
    },
    FunctionCallArguments {
        name: String,
        expected: usize,
        given: usize,
    },
    NativeFunctionWrongArgument {
        name: String,
        index: usize,
        expected: String,
        given: String,
    },
    NotIterable(String),
}
impl std::error::Error for InterpreterError {}
impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = Colors::error("[ERROR]");
        match self {
            InterpreterError::UnexpectedReturn => {
                return write!(f, "{} Unexpected return call.", prefix);
            }
            InterpreterError::DivBy0 => {
                return write!(f, "{} Division by 0 not permitted.", prefix);
            }
            InterpreterError::VarAlreadyDeclared(varname) => {
                return write!(
                    f,
                    "{} Cannot declare var '{}' : already defined in this scope.",
                    prefix, varname
                );
            }
            InterpreterError::VarUnresolvable(varname) => {
                return write!(f, "{} Unable to resolve var '{}'", prefix, varname);
            }
            InterpreterError::EditConst(varname) => {
                return write!(
                    f,
                    "{} Cannot assign const '{}'. By definition, a const cannot be edited.",
                    prefix, varname
                );
            }
            InterpreterError::UnknownBinaryOperator(operator) => {
                return write!(
                    f,
                    "{} Binary operation evaluation impossible : unknown operator '{}'.",
                    prefix, operator
                );
            }
            InterpreterError::UnpermittedBinaryOperation { lhs, rhs, operator } => {
                return write!(
                    f,
                    "{} Binary operation '{}' not permitted between values '{}' and '{}'.",
                    prefix, operator, lhs, rhs
                );
            }
            InterpreterError::UnknownUnaryOperator(operator) => {
                return write!(
                    f,
                    "{} Unary operation evaluation impossible : unknown operator '{}'.",
                    prefix, operator
                );
            }
            InterpreterError::UnknownLogicalOperator(operator) => {
                return write!(
                    f,
                    "{} Logical operation evaluation impossible : unknown operator '{}'.",
                    prefix, operator
                );
            }
            InterpreterError::NumberInterpretation(v) => {
                return write!(
                    f,
                    "{} Unsupported operation : {} cannot be interpreted as a number.",
                    prefix, v
                );
            }
            InterpreterError::Assignment(e) => {
                return write!(f, "{} Assignment error : unexpected {:?}.", prefix, e);
            }
            InterpreterError::ObjectKeyUncomputedNotIdentifier(e) => {
                return write!(
                    f,
                    "{} Access to an uncomputed property requires an identifier. {:?} is not an identifier.",
                    prefix, e
                );
            }
            InterpreterError::ObjectKeyComputedType(v) => {
                return write!(f, "{} Invalid type for object key : {:?}", prefix, v);
            }
            InterpreterError::MemberNotAccessible { action, value } => {
                return write!(
                    f,
                    "{} {} error : Cannot access a member of {}. Member access only available for objects / lists.",
                    prefix, action, value
                );
            }
            InterpreterError::InvalidIndex(i) => {
                return write!(f, "{} '{}' cannot be interpreted as an index", prefix, i);
            }
            InterpreterError::OutOfBounds(i) => {
                return write!(f, "{} Index '{}' is out of bounds.", prefix, i);
            }
            InterpreterError::NotAFunction { action, value } => {
                return write!(
                    f,
                    "{} Function {} error : {} is not a function.",
                    prefix, action, value
                );
            }
            InterpreterError::FunctionCallArguments {
                name,
                expected,
                given,
            } => {
                return write!(
                    f,
                    "{} Function call error : {} requires {} arguments, but {} were given.",
                    prefix, name, expected, given
                );
            }
            InterpreterError::NativeFunctionWrongArgument {
                name,
                index,
                expected,
                given,
            } => {
                return write!(
                    f,
                    "{} Native function call error : {} requires a {} as argument number {}, but recived {}",
                    prefix, name, expected, index, given
                );
            }
            InterpreterError::NotIterable(s) => {
                return write!(
                    f,
                    "{} Iteration impossible : {} is not iterable.",
                    prefix, s
                );
            }
        }
    }
}
impl InterpreterError {
    pub fn with_span(self, span: Span) -> RuntimeError {
        RuntimeError { error: self, span }
    }
}
