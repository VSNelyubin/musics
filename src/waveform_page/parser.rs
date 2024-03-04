use iced::{widget::{text_editor::{self, Content}, TextEditor}, Element};
use lalrpop_util::lalrpop_mod;
use std::{
    collections::{HashMap, HashSet},
    f32::consts::{E, PI},
};

use crate::MesDummies;

use super::ast::*;

lalrpop_mod!(pub formula_parser); // synthesized by LALRPOP

fn parse(s: &str) -> Vec<Statement> {
    use lalrpop_util::ParseError::{InvalidToken, UnrecognizedToken};

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

    let mut data = vec![0f32; 100];

    let pos = 50usize;
    let mouse_val = 10.;

    let mut var_registry: HashMap<String, f32> = HashMap::new();

    for stat in x {
        let val = stat.expr.eval(&data, &var_registry, (pos, mouse_val));
        match stat.target {
            Targets::ArrAcc(off) => data[summ(pos, off)] = val,
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
    fn eval(&self, data: &[f32], vars: &HashMap<String, f32>, mouse: (usize, f32)) -> f32 {
        match self {
            Expr::MouseVal => mouse.1,
            Expr::ArrAcc(off) => *data.get(summ(mouse.0, *off)).unwrap_or(&0.),
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
    string: String,
    content:Content,
}

impl Default for FormChild {
    fn default() -> Self {
        Self {
            string: "[0]=$m".to_string(),
            content:Content::new(),
        }
    }
}

impl FormChild {
    pub fn element(&self) -> Element<MesDummies> {
        let formula_edit = |act:iced::widget::text_editor::Action| match act{
            text_editor::Action::Move(_) => todo!(),
            text_editor::Action::Select(_) => todo!(),
            text_editor::Action::SelectWord => todo!(),
            text_editor::Action::SelectLine => todo!(),
            text_editor::Action::Edit(s) => match s{
                text_editor::Edit::Insert(_) => todo!(),
                text_editor::Edit::Paste(_) => todo!(),
                text_editor::Edit::Enter => todo!(),
                text_editor::Edit::Backspace => todo!(),
                text_editor::Edit::Delete => todo!(),
            },
            text_editor::Action::Click(_) => todo!(),
            text_editor::Action::Drag(_) => todo!(),
            text_editor::Action::Scroll { lines } => todo!(),
            
        // };MesDummies::WavePageSig {
        //     wp_sig: super::WavePageSig::FormulaChanged(string),
        };
        let formula_editor=TextEditor::new(&self.content).on_action(formula_edit);
        formula_editor.into()
    }
}

// pub fn parser_element() -> Element<'static, MesDummies> {
//     let formula_edit = |string: String| MesDummies::WavePageSig {
//         wp_sig: super::WavePageSig::FormulaChanged(string),
//     };
//     let formula_editor = text_input("[0]=m", "s[0]=m")
//         .width(256)
//         .on_input(formula_edit);
//     formula_editor.into()
// }
