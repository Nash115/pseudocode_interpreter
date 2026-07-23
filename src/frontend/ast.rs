#[derive(Debug, Clone)]
pub enum Expr {
    NumericLiteral(f64),
    Identifier(String),
    BinaryExpr {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: String,
    },
    ObjectLiteral(Vec<ObjectProperty>),
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

#[derive(Debug, Clone)]
pub enum Stmt {
    Program(Vec<Stmt>),
    VarDeclaration {
        constant: bool,
        identifier: String,
        value: Option<Expr>,
    },
    ExprStmt(Expr),
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub key: String,
    pub value: Option<Expr>,
}
