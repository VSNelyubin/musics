#![allow(unused)]

use iced::{
    advanced::graphics::text::cosmic_text::rustybuzz::ttf_parser::loca,
    widget::{
        column, text,
        text_editor::{self, Content, Edit},
        TextEditor,
    },
    Element,
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

fn parse(s: &str) -> Vec<Statement> {
    let rez = formula_parser::FormulaParser::new().parse(s);
    if let Err(e) = &rez {
        println!("{s}");
        match e {
            UnrecognizedToken {
                token: t,
                expected: ex,
            } => {
                for _ in 0..t.0 {
                    print!(" ")
                }
                println!("^\n{:?}\n{:?}", t, ex);
            }
            InvalidToken { location: t } => {
                for _ in 0..*t {
                    print!(" ")
                }
                println!("^\n{:?}", t);
            }
            ee => println!("{:#?}", ee),
        }
    }
    rez.unwrap_or_default()
}

// #[test]
// fn shmain() {
//     let s = "one = Min(1 , Avg( 2 , 3) , Sin(4) , &Mouse);";
//     let ss = "potato = (10+one);";
//     let sss = "potato = (one+two);";
//     let fin = format!("{s}{ss}{sss} w[0]=((potato*w[-1])*potato)");
//     let x = parse(&fin);
//     if !chek_all(&x) {
//         println!("use of undeclared variables");
//         return;
//     }
//     println!("{:#?}", x);

//     let mut data = vec![0i16; 100];

//     let pos = 50usize;
//     let mouse_val = 10.;

//     let mut var_registry: HashMap<String, f32> = HashMap::new();

//     for stat in x {
//         let val = stat.expr.eval(&data, &var_registry, (pos, mouse_val));
//         match stat.target {
//             Targets::ArrAcc(off) => data[summ(pos, off)] = val as i16,
//             Targets::Var(name) => {
//                 var_registry.insert(name, val);
//             }
//         }
//     }
// }

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
            let idx: usize = map
                .get(n)
                .cloned()
                .unwrap_or_else(|| map.insert(n.to_string(), map.len()).unwrap());
            e.target = Targets::ResVar(idx);
        }
    }
    Ok(map.len())
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
            e => e.clone(),
        })
    }
    fn eval(
        &self,
        data: (&[i16], usize),
        vars: &[f32],
        mouse: (usize, f32),
        selection: (usize, usize),
    ) -> f32 {
        match self {
            Expr::ArrAcc(off) => {
                let x: i16 = *data.0.get(summ(mouse.0 + data.1, *off)).unwrap_or(&0i16);
                x.into()
            }
            Expr::Var(name) => panic!("unresolved variable"), //*vars.get(name).unwrap_or(&0.),
            Expr::ResVar(idx) => vars[*idx],
            Expr::Spec(s) => match s {
                Spec::Pi => PI,
                Spec::E => E,
                Spec::Rnd => 0f32,
                Spec::Mouse => mouse.1,
                Spec::Time => (mouse.0 - selection.0) as f32,
                Spec::Fac => {
                    ((mouse.0 - selection.0) as f64 / (selection.1 - selection.0) as f64) as f32
                }
            },
            Expr::Float(f) => *f,
            Expr::Binary { l, o, r } => {
                let l = l.eval(data, vars, mouse, selection);
                let r = r.eval(data, vars, mouse, selection);
                match o {
                    Opr::Add => l + r,
                    Opr::Sub => l - r,
                    Opr::Mul => l * r,
                    Opr::Div => l / r,
                }
            }
            Expr::SaFunc { f, x } => {
                let v = x.eval(data, vars, mouse, selection);
                match f {
                    SaFunc::Sin => v.sin(),
                    SaFunc::Cos => v.cos(),
                    SaFunc::Tahn => v.tan(),
                    SaFunc::Abs => v.abs(),
                    SaFunc::Sign => v.signum(),
                }
            }
            Expr::MaFunc { f, xs } => {
                let vs = xs.iter().map(|x| x.eval(data, vars, mouse, selection));
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
                .unwrap_or(0.)
            }
        }
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
            formula: parse("[0]=&m"),
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
                return true;
            }
            let mut rez = rez.unwrap();

            match chek_all(&mut rez) {
                Err(s) => {
                    self.message1 = s.to_string();
                    self.message2 = "Undeclared variable".to_string();
                    self.formula = Vec::new();
                }
                Ok(s) => {
                    self.message1 = String::new();
                    self.message2 = "Success!".to_string();
                    self.formula = rez;
                    self.variables = 0;
                }
            }
        }
        should_parse
    }

    fn generate_err_message(&mut self, e: ParseError<usize, Token<'_>, &str>) {
        let (pos, msg) = match e {
            InvalidToken { location } => (location, "bad token"),
            ParseError::UnrecognizedEof { location, expected } => (location, "early EOF"),
            UnrecognizedToken { token, expected } => (token.0, "bad syntax"),
            ParseError::ExtraToken { token } => (token.0, "bad token"),
            ParseError::User { error } => unimplemented!(),
        };
        let rang = 10usize;
        let padd = vec!['_'; rang.saturating_sub(pos)];
        let string: String = padd
            .into_iter()
            .chain(self.content.text().chars())
            .skip(pos.saturating_sub(rang))
            .take(rang * 2 + 1)
            .map(|c| if c == '\n' { ' ' } else { c })
            .collect();
        self.message1 = string;
        let spaces: String = (0..rang).map(|_| '_').collect();
        self.message2 = format!("{spaces}^ - {msg}");
    }

    pub fn affect_data(
        &mut self,
        source: (&[i16], usize),
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
            match &stat.target {
                Targets::ArrAcc(off) => {
                    // println!("{:3} -> {:3}",data[summ(pos, *off)],val);
                    let position = summ(pos, *off);
                    if position < target.len() {
                        target[position] = val as i16
                    }
                }
                Targets::Var(name) => panic!("target wasnt resolved"),
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
        let er1 = text(self.message1.clone());
        let er2 = text(self.message2.clone());
        let wid = column!(formula_editor, er1, er2).spacing(pdd).padding(pdd);
        wid.into()
    }
}
