#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn null() -> Span {
        Span {
            start: Position { line: 0, col: 0 },
            end: Position { line: 0, col: 0 },
        }
    }
    pub fn is_null(&self) -> bool {
        if self.start.line == 0 && self.start.col == 0 && self.end.line == 0 && self.end.col == 0 {
            return true;
        }
        false
    }
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: if self.is_null() {
                other.start
            } else {
                self.start
            },
            end: if other.is_null() {
                self.start
            } else {
                other.start
            },
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}, Col {}", self.start.line, self.start.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}
