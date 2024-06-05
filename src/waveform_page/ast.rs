#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        target: Targets,
        expr: Expr,
    },
    Block(Box<Vec<Statement>>),
    Cond {
        ifs: Box<BinExpr>,
        thens: Box<Statement>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Targets {
    ArrAcc(i64),
    Var(String),
    ResVar(usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Opr {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Spec {
    Pi,
    E,
    Rnd,
    Mouse,
    Time,
    Fac,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SaFunc {
    Sin,
    Cos,
    Saw,
    Sqr,
    Tri,
    Tahn,
    Abs,
    Sign,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MaFunc {
    Min,
    Max,
    Avg,
    Med,
}

#[derive(Debug, Clone)]
pub enum Expr {
    ArrAcc(Box<Expr>),
    Var(String),
    ResVar(usize),
    Spec(Spec),
    Float(f32),
    Binary { l: Box<Expr>, o: Opr, r: Box<Expr> },
    SaFunc { f: SaFunc, x: Box<Expr> },
    MaFunc { f: MaFunc, xs: Box<Vec<Expr>> },
}

#[derive(Debug, Clone)]
pub enum BinExpr {
    Lit(bool),
    Cmp {
        l: Box<Expr>,
        o: CmpOpr,
        r: Box<Expr>,
    },
    Bin {
        l: Box<BinExpr>,
        o: LogOpr,
        r: Box<BinExpr>,
    },
}

#[derive(Debug, Clone)]
pub enum CmpOpr {
    Eq,
    Greq,
    Grt,
    Leq,
    Lss,
}

#[derive(Debug, Clone)]
pub enum LogOpr {
    And,
    Or,
    Xor,
}
