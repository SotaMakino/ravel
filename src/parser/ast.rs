use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    NumberLit(f64),
    StrLit(String),
    BoolLit(bool),
    NullLit,
    UndefinedLit,
    Ident(String),

    BinaryOp {
        left: Box<AstNode>,
        op: BinaryOp,
        right: Box<AstNode>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<AstNode>,
    },

    VarDecl {
        name: String,
        kind: VarKind,
        init: Option<Box<AstNode>>,
    },
    Assign {
        name: String,
        value: Box<AstNode>,
    },

    Block(Vec<AstNode>),
    If {
        cond: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    While {
        cond: Box<AstNode>,
        body: Box<AstNode>,
    },
    For {
        init: Option<Box<AstNode>>,
        cond: Option<Box<AstNode>>,
        update: Option<Box<AstNode>>,
        body: Box<AstNode>,
    },

    FuncDecl {
        name: String,
        params: Vec<String>,
        body: Box<AstNode>,
    },
    Call {
        callee: Box<AstNode>,
        args: Vec<AstNode>,
    },
    Return(Option<Box<AstNode>>),

    MemberAccess {
        object: Box<AstNode>,
        property: String,
    },
    IndexAccess {
        object: Box<AstNode>,
        index: Box<AstNode>,
    },
    ObjectLit(Vec<(String, AstNode)>),
    ArrayLit(Vec<AstNode>),

    Expr(Box<AstNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    StrictEq,
    NotEq,
    StrictNotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarKind {
    Let,
    Const,
    Var,
}

impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::NumberLit(n) => write!(f, "{}", n),
            AstNode::StrLit(s) => write!(f, "\"{}\"", s),
            AstNode::BoolLit(true) => write!(f, "true"),
            AstNode::BoolLit(false) => write!(f, "false"),
            AstNode::NullLit => write!(f, "null"),
            AstNode::UndefinedLit => write!(f, "undefined"),
            AstNode::Ident(s) => write!(f, "{}", s),
            AstNode::BinaryOp { left, op, right } => {
                write!(f, "({} {:?} {})", left, op, right)
            }
            AstNode::UnaryOp { op, operand } => {
                write!(f, "({:?} {})", op, operand)
            }
            AstNode::VarDecl { name, kind, init } => {
                let kind_str = match kind {
                    VarKind::Let => "let",
                    VarKind::Const => "const",
                    VarKind::Var => "var",
                };
                match init {
                    Some(i) => write!(f, "{} {} = {}", kind_str, name, i),
                    None => write!(f, "{} {}", kind_str, name),
                }
            }
            AstNode::Assign { name, value } => {
                write!(f, "{} = {}", name, value)
            }
            AstNode::Block(nodes) => {
                let parts: Vec<_> = nodes.iter().map(|n| format!("{}", n)).collect();
                write!(f, "{{ {} }}", parts.join("; "))
            }
            AstNode::If {
                cond,
                then_branch,
                else_branch,
            } => match else_branch {
                Some(e) => write!(f, "if {} {} else {}", cond, then_branch, e),
                None => write!(f, "if {} {}", cond, then_branch),
            },
            AstNode::While { cond, body } => {
                write!(f, "while {} {}", cond, body)
            }
            AstNode::For {
                init,
                cond,
                update,
                body,
            } => {
                let i = init.as_ref().map(|x| format!("{}", x)).unwrap_or_default();
                let c = cond.as_ref().map(|x| format!("{}", x)).unwrap_or_default();
                let u = update
                    .as_ref()
                    .map(|x| format!("{}", x))
                    .unwrap_or_default();
                write!(f, "for ({i}; {c}; {u}) {body}")
            }
            AstNode::FuncDecl { name, params, body } => {
                write!(f, "fn {}({}) {}", name, params.join(", "), body)
            }
            AstNode::Call { callee, args } => {
                let a: Vec<_> = args.iter().map(|a| format!("{}", a)).collect();
                write!(f, "{}({})", callee, a.join(", "))
            }
            AstNode::Return(Some(v)) => write!(f, "return {}", v),
            AstNode::Return(None) => write!(f, "return"),
            AstNode::MemberAccess { object, property } => {
                write!(f, "{}.{}", object, property)
            }
            AstNode::IndexAccess { object, index } => {
                write!(f, "{}[{}]", object, index)
            }
            AstNode::ObjectLit(entries) => {
                let parts: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{ {} }}", parts.join(", "))
            }
            AstNode::ArrayLit(items) => {
                let parts: Vec<_> = items.iter().map(|i| format!("{}", i)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            AstNode::Expr(e) => write!(f, "{}", e),
        }
    }
}
