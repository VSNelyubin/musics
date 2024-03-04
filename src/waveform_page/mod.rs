#![allow(unused_imports)]
pub mod ast;
pub mod drawer;
pub mod parser;
use iced::advanced::graphics::color;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse::{self, ScrollDelta};
use iced::widget::canvas::Cache;
use iced::widget::{button, column, row, text_input, vertical_rule, Canvas};
use iced::{Element, Length, Rectangle, Renderer, Size, Theme}; //, Vector, Point};

use rand::Rng;

use iced::widget::{Column, Row};

use crate::audio_player::play_i16_audio;
use crate::not_retarded_vector::NRVec;
use crate::MesDummies;

use self::drawer::{Transform, WaveDrawerSig, WaveformDrawer};

type Audi = i16;

const LIMIT: Audi = 256;

fn wrap(mut val: Audi) -> Audi {
    while val < -LIMIT {
        val += LIMIT;
    }
    while val > LIMIT {
        val -= LIMIT
    }
    val
}

#[derive(Default, Debug)]
pub struct WaveformPage {
    data: Vec<Audi>,
    selection: (usize, usize),
    transform: Transform,
    cache: Cache,
    sample_rate: u32,
    channels: u16,
    edit_mode: bool,
    edit_last_pos: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum WavePageSig {
    AllignSelect,
    FixNegScale,
    ToggleEditMode,
    ResetView,
    FormulaChanged(String),
}

impl WaveformPage {
    pub fn new_noisy(len: usize) -> Self {
        let mut rng = rand::thread_rng();
        let data = (0..len).map(|_| wrap(rng.gen::<Audi>())).collect();
        WaveformPage {
            data,
            transform: Transform::default(),
            cache: Cache::new(),
            sample_rate: 44800,
            channels: 1,
            selection: (0, 0),
            edit_mode: false,
            edit_last_pos: None,
        }
    }

    pub fn new_wedge(len: usize, focus: i16) -> Self {
        let data = (0..len)
            .map(|i| {
                // let factor: f32 = (i / len) as f32 - focus;
                // let val: i16 = ((i % 2) as i16 * 2 - 1) * LIMIT;
                // let fval: f32 = val.into();
                // (fval * factor).round() as i16
                let mut res = i.try_into().unwrap();
                res -= focus;
                if i % 2 == 0 {
                    res *= -1;
                }
                res
            })
            .collect();
        WaveformPage {
            data,
            transform: Transform::default(),
            cache: Cache::new(),
            sample_rate: 44800,
            channels: 1,
            selection: (0, 0),
            edit_mode: false,
            edit_last_pos: None,
        }
    }

    pub fn new_widh_data(data: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        WaveformPage {
            data,
            transform: Transform::default(),
            cache: Cache::new(),
            sample_rate,
            channels,
            selection: (0, 0),
            edit_mode: false,
            edit_last_pos: None,
        }
    }

    pub fn append_noise(&mut self, len: usize) {
        let mut rng = rand::thread_rng();
        self.data.extend((0..len).map(|_| wrap(rng.gen::<Audi>())))
    }
}

impl WaveformPage {
    fn scroll(&mut self, delta: ScrollDelta) {
        self.transform.scroll(delta);
    }

    fn scale(&mut self, scale: NRVec) {
        self.transform.scale(scale);
    }

    fn select_begin(&mut self, begin: NRVec) {
        let rounded = self.transform.get_pos(begin.x);
        self.selection.0 = rounded;
    }

    fn select_end(&mut self, begin: NRVec) {
        let rounded = self.transform.get_pos(begin.x);
        self.selection.1 = rounded.min(self.data.len() - 1);
    }

    fn fix_select(&mut self) {
        if self.selection.0 > self.selection.1 {
            self.selection = (self.selection.1, self.selection.0);
        }
    }

    fn edit(&mut self, mid: NRVec) {
        let pos = self.transform.get_pos(mid.x);
        let pos = pos.min(self.selection.1).max(self.selection.0);
        self.write_data(pos, self.transform.get_amp(mid.y));
        if let Some(pos_old) = self.edit_last_pos {
            let (begin, end) = if pos_old < pos {
                (pos_old, pos)
            } else {
                (pos, pos_old)
            };
            // println!("{start} - {end}");
            for (fac, i_pos) in (begin..end).enumerate().skip(1) {
                let factor = fac as f32 / (end - begin) as f32;
                let factor = if factor.is_nan() { 1.0 } else { factor };
                // print!("{:1.2} ",factor);
                // print!("{:2} ", fac);
                let val = self.data[end] as f32 * factor + self.data[begin] as f32 * (1.0 - factor);
                // let val = self.transform.get_amp(mid.y);
                self.write_data(i_pos, val as i16);
            }
            // println!();
        }
        self.edit_last_pos = Some(pos);
    }

    fn write_data(&mut self, pos: usize, sample: i16) {
        if self.selection.0 <= pos && pos <= self.selection.1 {
            self.data[pos] = sample;
        }
    }

    fn request_redraw(&mut self) {
        self.cache.clear();
    }

    fn drawer(&self) -> WaveformDrawer {
        WaveformDrawer::new(self)
    }

    fn side_menu(&self) -> Element<'static, MesDummies> {
        let allign_select = MesDummies::WavePageSig {
            wp_sig: WavePageSig::AllignSelect,
        };
        let fix_neg_scale = MesDummies::WavePageSig {
            wp_sig: WavePageSig::FixNegScale,
        };
        let toggle_edit = MesDummies::WavePageSig {
            wp_sig: WavePageSig::ToggleEditMode,
        };
        let reset_view = MesDummies::WavePageSig {
            wp_sig: WavePageSig::ResetView,
        };
        // let formula_edit = |string: String| MesDummies::WavePageSig {
        //     wp_sig: WavePageSig::FormulaChanged(string),
        // };
        let pdd = 5;
        let but_reset_view = button("Reset View").padding(pdd).on_press(reset_view);
        let but_allign = button("Allign to select")
            .padding(pdd)
            .on_press_maybe((self.selection.0 != self.selection.1).then_some(allign_select));
        let but_fix_negative = button("Flip negative scale")
            .padding(pdd)
            .on_press(fix_neg_scale);
        let but_edit_toggle = button(if self.edit_mode {
            "Select mode"
        } else {
            "Edit Mode"
        })
        .padding(pdd)
        .on_press(toggle_edit);
        // let formula_editor = text_input("[0]=m", "s[0]=m")
        //     .width(256)
        //     .on_input(formula_edit);
        let formula_editor = parser::parser_element();
        let menu = column![
            but_reset_view,
            but_allign,
            but_fix_negative,
            but_edit_toggle,
            formula_editor
        ]
        .spacing(pdd)
        .padding(pdd)
        .width(Length::Shrink);
        menu.into()
    }

    // pub fn view<'a>(&'a self) -> Element<'a, MesDummies> {
    pub fn view(&self) -> Element<'_, MesDummies> {
        let elem = Canvas::new(self.drawer())
            .width(Length::Fill)
            .height(Length::Fill);
        let rez = row![self.side_menu(), vertical_rule(5), elem];
        rez.into()
    }

    pub fn play_audio(&self) {
        play_i16_audio(&self.data, self.sample_rate, self.channels);
    }

    pub fn process_wave_drawer_sig(&mut self, signal: WaveDrawerSig) {
        use WaveDrawerSig::*;
        match signal {
            Scroll { delta } => {
                self.scroll(delta);
                self.request_redraw();
            }
            ResizeBegin { begin } => println!("resize begin from {:?}", begin),
            ResizeEnd { end } => println!("resize  end   at  {:?}", end),

            Resize { scale } => {
                self.scale(scale);
                self.request_redraw();
            }

            SelectOrEditBegin { begin } => {
                if !self.edit_mode {
                    self.select_begin(begin);
                    // println!("select begin from {:?}", begin)
                    self.request_redraw();
                } else {
                    self.edit_last_pos = None;
                }
            }
            SelectOrEdit { mid } => {
                if self.edit_mode {
                    self.edit(mid);
                } else {
                    self.select_end(mid);
                }
                self.request_redraw();
            }
            SelectOrEditEnd { end } => {
                if !self.edit_mode {
                    self.select_end(end);
                    self.fix_select();
                    // println!("select  end   at  {:?}", end);
                    self.request_redraw();
                } else {
                    self.edit_last_pos = None;
                }
            }

            ForceRedraw => self.request_redraw(),
        }
    }

    pub fn process_page_signal(&mut self, signal: WavePageSig) {
        use WavePageSig::*;
        match signal {
            AllignSelect => self.transform.allign_select(self.selection),
            FixNegScale => self.transform.fix_negative(),
            ToggleEditMode => self.edit_mode = !self.edit_mode,
            ResetView => self.transform = Transform::default(),
            FormulaChanged(_string) => (),
        }
        self.request_redraw();
    }
}
