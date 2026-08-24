#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    NumericLiteral(f64),
    StringLiteral(String),
    Identifier(String),
    BinaryExpr {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: String,
    },
    UnaryExpr {
        right: Box<Expr>,
        operator: String,
    },
    LogicalExpr {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: String,
    },
    ObjectLiteral(Vec<ObjectProperty>),
    ListLiteral(Vec<Expr>),
    AssignmentExpr {
        assigne: Box<Expr>,
        value: Box<Expr>,
    },
    MemberExpr {
        object: Box<Expr>,
        property: Box<Expr>,
        computed: bool,
    },
    CallExpr {
        args: Vec<Expr>,
        caller: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Program(Vec<Stmt>),
    VarDeclaration {
        constant: bool,
        identifier: String,
        value: Option<Expr>,
    },
    FnDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Expr),
    Condition {
        test: Expr,
        body: Vec<Stmt>,
        alternate: Option<Vec<Stmt>>,
    },
    WhileLoop {
        test: Expr,
        body: Vec<Stmt>,
    },
    ForLoop {
        iterable: Expr,
        identifier: String,
        body: Vec<Stmt>,
    },
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
    pub key: String,
    pub value: Option<Expr>,
}
