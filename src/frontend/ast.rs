use crate::frontend::span::Spanned;

pub type ExprNode = Spanned<Expr>;
pub type StmtNode = Spanned<Stmt>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    NumericLiteral(f64),
    StringLiteral(String),
    Identifier(String),
    BinaryExpr {
        left: Box<ExprNode>,
        right: Box<ExprNode>,
        operator: String,
    },
    UnaryExpr {
        right: Box<ExprNode>,
        operator: String,
    },
    LogicalExpr {
        left: Box<ExprNode>,
        right: Box<ExprNode>,
        operator: String,
    },
    ObjectLiteral(Vec<ObjectProperty>),
    ListLiteral(Vec<ExprNode>),
    AssignmentExpr {
        assigne: Box<ExprNode>,
        value: Box<ExprNode>,
    },
    MemberExpr {
        object: Box<ExprNode>,
        property: Box<ExprNode>,
        computed: bool,
    },
    CallExpr {
        args: Vec<ExprNode>,
        caller: Box<ExprNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Program(Vec<StmtNode>),
    VarDeclaration {
        constant: bool,
        identifier: String,
        value: Option<ExprNode>,
    },
    FnDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<StmtNode>,
    },
    Return(ExprNode),
    Condition {
        test: ExprNode,
        body: Vec<StmtNode>,
        alternate: Option<Vec<StmtNode>>,
    },
    WhileLoop {
        test: ExprNode,
        body: Vec<StmtNode>,
    },
    ForLoop {
        iterable: ExprNode,
        identifier: String,
        body: Vec<StmtNode>,
    },
    ExprStmt(ExprNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
    pub key: String,
    pub value: Option<ExprNode>,
}
