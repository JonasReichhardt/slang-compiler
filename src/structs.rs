use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(i64),
    Char(char),

    // keywords
    Var,
    Fn,
    If,
    ElseIf,
    Else,
    While,
    Return,

    // symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Semicolon,
    Comma,

    Assign, // =
    Neq,    // #
    Lt,
    Gt,
    Le,
    Ge,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    EOF,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Char(char),
    Call(String, Vec<Expr>),
    Ident {
        name: String,
        loc: Option<VarLocation>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl fmt::Display for BinaryOp {
    #[rustfmt::skip]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {

        let name = match self {
            BinaryOp::Add => "add",BinaryOp::Sub=>"sub",BinaryOp::Div=>"div",
            BinaryOp::Mul=>"mul",BinaryOp::Mod=>"rem",
        };

        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Char,
    Void,
    Error,
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        match value.as_str() {
            "int" => Type::Int,
            "char" => Type::Char,
            "void" => Type::Void,
            _ => Type::Error,
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Call(String, Vec<Expr>),
    Assign {
        name: String,
        loc: Option<VarLocation>,
        expr: Expr,
    },
    If {
        branches: Vec<(Condition, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        cond: Condition,
        body: Vec<Statement>,
    },
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarLocation {
    Stack(i32),
    Global(String),
}

#[derive(Debug)]
pub enum Declaration {
    Var(VarDecl),
    Fn(FuncDecl),
}

#[derive(Debug)]
pub struct VarDecl {
    pub name: String,
    pub typ: Type,
    pub loc: Option<VarLocation>,
}

#[derive(Debug)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub locals: Vec<VarDecl>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub left: Expr,
    pub op: RelOp,
    pub right: Expr,
}
