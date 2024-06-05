use iced::{
    advanced::graphics::text::cosmic_text::rustybuzz::ttf_parser::loca,
    widget::{
        column, text,
        text_editor::{self, Content, Edit},
        TextEditor,
    },
    Element, Font,
};
use lalrpop_util::{lalrpop_mod, lexer::Token, ParseError};
use std::{
    collections::{HashMap, HashSet},
    f32::consts::{E, PI},
};

use crate::MesDummies;
use lalrpop_util::ParseError::{InvalidToken, UnrecognizedToken};

use super::ast::*;

lalrpop_mod!(pub formula_parser); // synthesized by LALRPOP

fn summ(a: usize, b: i64) -> usize {
    if b < 0 {
        a.saturating_sub(b.unsigned_abs() as usize)
    } else {
        a.saturating_add(b as usize)
    }
}

/// replace all string vars with indexes, or find the first one that doesnt resolve
fn chek_all(exprs: &mut [Statement]) -> Result<usize, String> {
    let mut map = HashMap::new();
    for e in exprs.iter_mut() {
        e.expr = e.expr.checkonvert(&map)?;
        if let Targets::Var(n) = &e.target {
            let idx: usize = map.get(n).cloned().unwrap_or_else(|| {
                let v = map.len();
                map.insert(n.to_string(), v);
                v
            });
            e.target = Targets::ResVar(idx);
        }
    }
    Ok(map.len())
}

#[derive(Debug, Clone, Copy)]
pub enum EvalErr {
    DivByZero,
    NoArgs,
}

impl Expr {
    /// replace string vars with indexes, or tell which vars are undeclared
    fn checkonvert(&self, vars: &HashMap<String, usize>) -> Result<Self, String> {
        Ok(match self {
            Expr::Var(v) => match vars.get(v) {
                Some(&n) => Self::ResVar(n),
                None => return Err(v.clone()),
            },
            Expr::Binary { l, r, o } => {
                let ll = l.checkonvert(vars)?;
                let rr = r.checkonvert(vars)?;
                Expr::Binary {
                    l: Box::new(ll),
                    o: *o,
                    r: Box::new(rr),
                }
            }
            Expr::SaFunc { x, f } => Expr::SaFunc {
                x: Box::new(x.checkonvert(vars)?),
                f: *f,
            },
            Expr::MaFunc { xs, f } => {
                let xs: Result<Vec<Expr>, String> =
                    xs.iter().map(|x| x.checkonvert(vars)).collect();
                Expr::MaFunc {
                    f: *f,
                    xs: Box::new(xs?),
                }
            }
            Expr::ArrAcc(e) => {
                let ee = e.checkonvert(vars)?;
                Expr::ArrAcc(Box::new(ee))
            }
            e => e.clone(),
        })
    }
    fn eval(
        &self,
        data: &[i16],
        vars: &[f32],
        mouse: (usize, f32),
        selection: (usize, usize),
    ) -> Result<f32, EvalErr> {
        let rez = match self {
            Expr::ArrAcc(off) => {
                let offset = off.eval(data, vars, mouse, selection)?;
                let off_i64 = offset.floor();
                let fac = offset - off_i64;
                let off_i64 = off_i64 as i64;
                let xl: i16 = *data
                    .get(summ(mouse.0 + selection.0, off_i64))
                    .unwrap_or(&0i16);
                let xr: i16 = *data
                    .get(summ(mouse.0 + selection.0, off_i64 + 1))
                    .unwrap_or(&0i16);
                assert!((0.0..=1.0).contains(&fac));
                let xl: f32 = xl.into();
                let xr: f32 = xr.into();
                xl * (1.0 - fac) + xr * fac
            }
            Expr::Var(_) => panic!("unresolved variable"), //*vars.get(name).unwrap_or(&0.),
            Expr::ResVar(idx) => vars[*idx],
            Expr::Spec(s) => match s {
                Spec::Pi => PI,
                Spec::E => E,
                Spec::Rnd => 0f32,
                Spec::Mouse => mouse.1,
                Spec::Time => (mouse.0 + selection.0) as f32,
                Spec::Fac => (mouse.0 as f64 / (selection.1 - selection.0) as f64) as f32,
            },
            Expr::Float(f) => *f,
            Expr::Binary { l, o, r } => {
                let l = l.eval(data, vars, mouse, selection)?;
                let r = r.eval(data, vars, mouse, selection)?;
                match o {
                    Opr::Add => l + r,
                    Opr::Sub => l - r,
                    Opr::Mul => l * r,
                    Opr::Div => {
                        let tmp = l / r;
                        if !tmp.is_finite() {
                            return Err(EvalErr::DivByZero);
                        }
                        tmp
                    }
                }
            }
            Expr::SaFunc { f, x } => {
                let v = x.eval(data, vars, mouse, selection)?;
                match f {
                    SaFunc::Sin => v.sin(),
                    SaFunc::Cos => v.cos(),
                    SaFunc::Tahn => v.tan(),
                    SaFunc::Abs => v.abs(),
                    SaFunc::Sign => v.signum(),
                    SaFunc::Saw => ((v / PI) - (v / PI).floor()) * 2. - 1.,
                    SaFunc::Sqr => v.sin().signum(),
                    SaFunc::Tri => {
                        let tmp = (v / PI) - (v / PI).floor();
                        (tmp * 2.).min(2. - tmp * 2.) * 2. - 1.
                    }
                }
            }
            Expr::MaFunc { f, xs } => {
                let vs = xs.iter().map(|x| x.eval(data, vars, mouse, selection));
                if let Some(x) = vs.clone().find(|x| x.is_err()) {
                    x?;
                }
                let vs = vs.map(|x| x.unwrap());
                match f {
                    MaFunc::Min => vs.fold(None, |a, i| {
                        Some(if let Some(a) = a { i.min(a) } else { i })
                    }),
                    MaFunc::Max => vs.fold(None, |a, i| {
                        Some(if let Some(a) = a { i.max(a) } else { i })
                    }),
                    MaFunc::Avg => {
                        let len = xs.len() as f32;
                        vs.fold(None, |a, i| Some(if let Some(a) = a { i + a } else { i }))
                            .map(|sum| sum / len)
                    }
                    MaFunc::Med => {
                        let op: Option<(f32, f32)> = vs.clone().fold(None, |pair, i| {
                            Some(if let Some((min, max)) = pair {
                                (i.min(min), i.max(max))
                            } else {
                                (i, i)
                            })
                        });
                        if let Some((l, g)) = op {
                            Some((l + g) / 2.)
                        } else {
                            None
                        }
                    }
                }
                .ok_or(EvalErr::NoArgs)?
            }
        };
        if !rez.is_finite() {
            panic!("bad number");
        }
        Ok(rez)
    }
}

impl Statement{
    fn execute(
        &self,
        source: &[i16],
        target: &mut [i16],
        mouse: (usize, f32),
        selection: (usize, usize),
        var_num:usize,
    ) {
        let (pos, mouse_val) = mouse;
        let mut variables = vec![0.; var_num];
        match self
    }
}

#[derive(Debug)]
pub struct FormChild {
    content: Content,
    formula: Vec<Statement>,
    message1: String,
    message2: String,
    variables: usize,
}

impl Default for FormChild {
    fn default() -> Self {
        Self {
            content: Content::with_text("[0]=&m"),
            formula: formula_parser::FormulaParser::new()
                .parse("[0]=&m")
                .unwrap(),
            message1: "".to_string(),
            message2: "Ok".to_string(),
            variables: 0,
        }
    }
}

#[test]
fn scans() {
    for i in 0..10 {
        print!("{:2} ", i);
    }
    println!();
    for i in (0..10).scan(0, |a, i| {
        *a += i;
        Some(*a)
    }) {
        print!("{:2} ", i);
    }
    println!();
}

impl FormChild {
    pub fn act(&mut self, act: text_editor::Action) -> bool {
        let should_parse = matches!(&act, text_editor::Action::Edit(_));
        self.content.perform(act);
        if should_parse {
            let text: &str = &self.content.text();
            let rez = formula_parser::FormulaParser::new().parse(text);
            if let Err(e) = rez {
                self.generate_err_message(e);
                return false;
            }
            let mut rez = rez.unwrap();

            return match chek_all(&mut rez) {
                Err(s) => {
                    self.message1 = s.to_string();
                    self.message2 = "Undeclared variable".to_string();
                    self.formula = Vec::new();
                    false
                }
                Ok(s) => {
                    self.message1 = String::new();
                    self.message2 = "Success!".to_string();
                    self.formula = rez;
                    self.variables = s;
                    true
                }
            };
        }
        false
    }

    fn generate_err_message(&mut self, e: ParseError<usize, Token<'_>, &str>) {
        let (pos, msg) = match e {
            InvalidToken { location } => (location, "bad token"),
            ParseError::UnrecognizedEof { location, .. } => (location, "early EOF"),
            UnrecognizedToken { token, .. } => (token.0, "bad syntax"),
            ParseError::ExtraToken { token } => (token.0, "bad token"),
            ParseError::User { .. } => unimplemented!(),
        };
        let rang = 10usize;
        let padd = vec![' '; rang.saturating_sub(pos)];
        let string: String = padd
            .into_iter()
            .chain(self.content.text().chars())
            .skip(pos.saturating_sub(rang))
            .take(rang * 2 + 1)
            .map(|c| if c == '\n' { ' ' } else { c })
            .collect();
        self.message1 = string;
        let spaces: String = (0..rang).map(|_| ' ').collect();
        self.message2 = format!("{spaces}^ - {msg}");
    }

    fn eval_error_message(&mut self, e: EvalErr) {
        match e {
            EvalErr::DivByZero => {
                self.message1 = "Sample skipped".to_string();
                self.message2 = "Divide by Zero occured".to_string()
            }
            EvalErr::NoArgs => {
                self.message1 = String::new();
                self.message2 = "No arguments provided".to_string()
            }
        }
    }

    pub fn affect_data(
        &mut self,
        source: &[i16],
        target: &mut [i16],
        mouse: (usize, f32),
        selection: (usize, usize),
    ) {
        let (pos, mouse_val) = mouse;
        let mut variables = vec![0.; self.variables];

        for stat in &self.formula {
            let val = stat
                .expr
                .eval(source, &variables, (pos, mouse_val), selection);
            let val = match val {
                Ok(val) => val,
                Err(e) => {
                    self.eval_error_message(e);
                    return;
                }
            };
            match &stat.target {
                Targets::ArrAcc(off) => {
                    // println!("{:3} -> {:3}",data[summ(pos, *off)],val);
                    let position = summ(pos, *off);
                    if position < target.len() {
                        target[position] = val as i16
                    }
                }
                Targets::Var(_) => panic!("target wasnt resolved"),
                Targets::ResVar(i) => variables[*i] = val,
            }
        }
    }

    pub fn element(&self) -> Element<MesDummies> {
        let formula_edit = |act: iced::widget::text_editor::Action| MesDummies::WavePageSig {
            wp_sig: super::WavePageSig::FormulaChanged(act),
        };
        let pdd = 5;
        let formula_editor = TextEditor::new(&self.content).on_action(formula_edit);
        let er1 = text(self.message1.clone()).font(Font::MONOSPACE);
        let er2 = text(self.message2.clone()).font(Font::MONOSPACE);
        let wid = column!(formula_editor, er1, er2).spacing(pdd).padding(pdd);
        wid.into()
    }
}
