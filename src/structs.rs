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

#[derive(Debug)]
pub enum Expr {
    Number(i64),
    Char(char),
    Ident(String),
    Call(String, Vec<Expr>),

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

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug)]
pub enum Statement {
    Assign(String, Expr),
    Call(String, Vec<Expr>),
    If {
        branches: Vec<(Expr, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        cond: Expr,
        body: Vec<Statement>,
    },
    Return(Option<Expr>),
}

#[derive(Debug)]
pub enum Declaration {
    Var(String, String),
    Fn {
        name: String,
        params: Vec<(String, String)>,
        ret: Option<String>,
        locals: Vec<(String, String)>,
        body: Vec<Statement>,
    },
}
