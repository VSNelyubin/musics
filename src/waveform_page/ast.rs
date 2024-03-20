#[derive(Debug, Clone)]
pub struct Statement {
    pub target: Targets,
    pub expr: Expr,
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
