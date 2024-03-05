#[derive(Debug, Clone)]
pub struct Statement {
    pub target: Targets,
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Targets {
    ArrAcc(i64),
    Var(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SaFunc {
    Sin,
    Cos,
    Tahn,
    Abs,
    Sign,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MaFunc {
    Min,
    Max,
    Avg,
    Med,
}

#[derive(Debug, Clone)]
pub enum Expr {
    MouseVal,
    ArrAcc(i64),
    Var(String),
    Spec(Spec),
    Float(f32),
    Binary { l: Box<Expr>, o: Opr, r: Box<Expr> },
    SaFunc { f: SaFunc, x: Box<Expr> },
    MaFunc { f: MaFunc, xs: Box<Vec<Expr>> },
}
