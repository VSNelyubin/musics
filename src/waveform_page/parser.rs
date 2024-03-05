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

#[test]
fn shmain() {
    let s = "one = Min(1 , Avg( 2 , 3) , Sin(4) , &Mouse);";
    let ss = "potato = (10+one);";
    let sss = "potato = (one+two);";
    let fin = format!("{s}{ss}{sss} w[0]=((potato*w[-1])*potato)");
    let x = parse(&fin);
    if !chek_all(&x) {
        println!("use of undeclared variables");
        return;
    }
    println!("{:#?}", x);

    let mut data = vec![0i16; 100];

    let pos = 50usize;
    let mouse_val = 10.;

    let mut var_registry: HashMap<String, f32> = HashMap::new();

    for stat in x {
        let val = stat.expr.eval(&data, &var_registry, (pos, mouse_val));
        match stat.target {
            Targets::ArrAcc(off) => data[summ(pos, off)] = val as i16,
            Targets::Var(name) => {
                var_registry.insert(name, val);
            }
        }
    }
}

fn summ(a: usize, b: i64) -> usize {
    if b < 0 {
        a.saturating_sub(b.unsigned_abs() as usize)
    } else {
        a.saturating_add(b as usize)
    }
}

fn chek_all(exprs: &[Statement]) -> bool {
    exprs
        .iter()
        .scan(HashSet::new(), |a, i| {
            let ans = i.expr.check_vars(a);
            if let Targets::Var(s) = &i.target {
                a.insert(s.to_string());
            }
            Some(ans)
        })
        .all(|x| x)
}

impl Expr {
    fn check_vars(&self, vars: &HashSet<String>) -> bool {
        match self {
            Expr::Var(v) => vars.contains(v),
            Expr::Binary { l, r, .. } => l.check_vars(vars) && r.check_vars(vars),
            Expr::SaFunc { x, .. } => x.check_vars(vars),
            Expr::MaFunc { xs, .. } => xs.iter().all(|x| x.check_vars(vars)),
            _ => true,
        }
    }
    fn eval(&self, data: &[i16], vars: &HashMap<String, f32>, mouse: (usize, f32)) -> f32 {
        match self {
            Expr::MouseVal => mouse.1,
            Expr::ArrAcc(off) => {
                let x: i16 = *data.get(summ(mouse.0, *off)).unwrap_or(&0i16);
                x.into()
            }
            Expr::Var(name) => *vars.get(name).unwrap_or(&0.),
            Expr::Spec(s) => match s {
                Spec::Pi => PI,
                Spec::E => E,
                Spec::Rnd => 0f32,
            },
            Expr::Float(f) => *f,
            Expr::Binary { l, o, r } => {
                let l = l.eval(data, vars, mouse);
                let r = r.eval(data, vars, mouse);
                match o {
                    Opr::Add => l + r,
                    Opr::Sub => l - r,
                    Opr::Mul => l * r,
                    Opr::Div => l / r,
                }
            }
            Expr::SaFunc { f, x } => {
                let v = x.eval(data, vars, mouse);
                match f {
                    SaFunc::Sin => v.sin(),
                    SaFunc::Cos => v.cos(),
                    SaFunc::Tahn => v.tan(),
                    SaFunc::Abs => v.abs(),
                    SaFunc::Sign => v.signum(),
                }
            }
            Expr::MaFunc { f, xs } => {
                let vs = xs.iter().map(|x| x.eval(data, vars, mouse));
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
}

impl Default for FormChild {
    fn default() -> Self {
        Self {
            content: Content::with_text("[0]=&m"),
            formula: parse("[0]=&m"),
            message1: "".to_string(),
            message2: "Ok".to_string(),
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
    pub fn act(&mut self, act: text_editor::Action) {
        let should_parse = matches!(&act, text_editor::Action::Edit(_));
        self.content.perform(act);
        if should_parse {
            let text: &str = &self.content.text();
            let rez = formula_parser::FormulaParser::new().parse(text);
            if let Err(e) = rez {
                self.generate_err_message(e);
                return;
            }
            let rez = rez.unwrap();

            if !chek_all(&rez) {
                self.message1 = String::new();
                self.message2 = "Undeclared variables".to_string();
                self.formula = Vec::new();
            } else if !rez.is_empty() {
                self.message1 = String::new();
                self.message2 = "Success!".to_string();
                self.formula = rez;
            }
        }
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

    pub fn affect_data(&self, data: &mut [i16], mouse: (usize, f32)) {
        let (pos, mouse_val) = mouse;
        let mut var_registry: HashMap<String, f32> = HashMap::new();

        for stat in &self.formula {
            let val = stat.expr.eval(data, &var_registry, (pos, mouse_val));
            match &stat.target {
                Targets::ArrAcc(off) => {
                    // println!("{:3} -> {:3}",data[summ(pos, *off)],val);
                    data[summ(pos, *off)] = val as i16
                }
                Targets::Var(name) => {
                    var_registry.insert(name.to_string(), val);
                }
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
