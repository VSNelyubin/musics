#![allow(unused_imports)]
pub mod ast;
pub mod drawer;
mod parser;
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
use crate::wav_writer::save_wav;
use crate::MesDummies;

use self::drawer::{Transform, WaveDrawerSig, WaveformDrawer};
use self::parser::FormChild;

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
    edit_buffer: Vec<Option<Audi>>,
    affected_data: Vec<Audi>,
    transform: Transform,
    cache: Cache,
    sample_rate: u32,
    channels: u16,
    edit_mode: bool,
    edit_last_pos: Option<usize>,
    parser: parser::FormChild,
}

#[derive(Debug, Clone)]
pub enum WavePageSig {
    AllignSelect,
    FixNegScale,
    ToggleEditMode { save: bool },
    ResetView,
    Cut { delete: bool },
    Copy,
    Paste { empty: Option<usize> },
    FormulaChanged(iced::widget::text_editor::Action),
}

// misc functions
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
            edit_buffer: Vec::new(),
            affected_data: Vec::new(),
            edit_mode: false,
            edit_last_pos: None,
            parser: FormChild::default(),
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
            selection: (0, len / 2),
            edit_buffer: Vec::new(),
            affected_data: Vec::new(),
            edit_mode: false,
            edit_last_pos: None,
            parser: FormChild::default(),
        }
    }

    pub fn new_widh_data(data: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        WaveformPage {
            transform: Transform::default(),
            // transform: Transform::new(
            //     NRVec {
            //         x:  (sample_rate as f32),
            //         y:  (*data
            //                 .iter()
            //                 .take(sample_rate.try_into().unwrap())
            //                 .max()
            //                 .unwrap() as f32),
            //     },
            //     0,
            // ),
            data,
            cache: Cache::new(),
            sample_rate,
            channels,
            selection: (0, 0),
            edit_buffer: Vec::new(),
            affected_data: Vec::new(),
            edit_mode: false,
            edit_last_pos: None,
            parser: FormChild::default(),
        }
    }

    pub fn append_noise(&mut self, len: usize) {
        let mut rng = rand::thread_rng();
        self.data.extend((0..len).map(|_| wrap(rng.gen::<Audi>())))
    }

    pub fn save_wav(&self) {
        save_wav(&self.data, self.sample_rate, self.channels)
    }

    pub fn focus_data(&self) -> Vec<i16> {
        if self.edit_mode {
            self.affected_data.clone()
        } else {
            self.data[self.selection.0..self.selection.1].to_vec()
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn select_len(&self) -> usize {
        self.selection.1.max(self.selection.0) - self.selection.0.min(self.selection.1)
    }
}

// model functions
impl WaveformPage {
    fn scroll(&mut self, delta: ScrollDelta) {
        self.transform.scroll(delta, self.data.len());
    }

    fn scale(&mut self, scale: NRVec) {
        self.transform.scale(scale);
    }

    fn select_begin(&mut self, begin: NRVec) {
        let rounded = self.transform.get_pos(begin.x);
        self.selection.0 = rounded.min(self.data.len() - 1);
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
        self.edit_buffer[pos - self.selection.0] = Some(self.transform.get_amp(mid.y));
        // self.edit_buffer[pos - self.selection.0] = Some(self.transform.get_amp(mid.y));
        if let Some(pos_old) = self.edit_last_pos {
            self.smooth_data(pos_old.min(pos), pos_old.max(pos));

            self.calc_data(pos_old.min(pos), pos_old.max(pos), false)
        } else {
            self.calc_data(pos, pos, false)
        }
        self.edit_last_pos = Some(pos);
    }

    fn erase(&mut self, mid: NRVec) {
        let pos = self.transform.get_pos(mid.x);
        let pos = pos.min(self.selection.1).max(self.selection.0);
        self.edit_buffer[pos - self.selection.0] = None;
        if let Some(pos_old) = self.edit_last_pos {
            let (begin, end) = if pos_old < pos {
                (pos_old, pos)
            } else {
                (pos, pos_old)
            };
            for i_pos in begin..end {
                self.edit_buffer[i_pos - self.selection.0] = None;
            }
            self.calc_data(begin, end, true)
        } else {
            self.calc_data(pos, pos, true)
        }
        self.edit_last_pos = Some(pos);
    }

    fn smooth_data(&mut self, begin: usize, end: usize) {
        let (begin, end) = (begin - self.selection.0, end - self.selection.0);
        for (fac, i_pos) in (begin..end).enumerate().skip(1) {
            let factor = fac as f32 / (end - begin) as f32;
            let factor = if factor.is_nan() { 1.0 } else { factor };
            let val = self.edit_buffer[end].unwrap_or_default() as f32 * factor
                + self.edit_buffer[begin].unwrap_or_default() as f32 * (1.0 - factor);
            self.edit_buffer[i_pos] = Some(val as i16);
        }
    }

    fn calc_data(&mut self, begin: usize, end: usize, reset: bool) {
        let (begin, end) = (begin - self.selection.0, end - self.selection.0);
        // self.affected_data = (self.selection.0..=self.selection.1)
        //     .map(|i| self.data[i])
        //     .collect();
        for i in self
            .edit_buffer
            .iter()
            .enumerate()
            .skip(begin)
            .take(end - begin + 1)
        {
            if reset {
                self.affected_data[i.0] = self.data[i.0 + self.selection.0]
            } else if let Some(sam) = i.1 {
                self.parser.affect_data(
                    &self.data,
                    &mut self.affected_data,
                    (i.0, *sam as f32),
                    self.selection,
                );
            }
        }
    }

    fn cut_data(&mut self) -> Vec<i16> {
        let mut tail = self.data.split_off(self.selection.1 + 1);
        let mid = self.data.split_off(self.selection.0);
        self.data.append(&mut tail);
        self.affected_data = Vec::new();
        self.edit_buffer = Vec::new();
        self.selection.1 = self.selection.0;
        mid
    }

    fn copy_data(&self) -> Vec<i16> {
        let rez = self
            .data
            .iter()
            .take(self.selection.1 + 1)
            .skip(self.selection.0)
            .cloned()
            .collect(); //?????????
        rez
    }

    fn insert_data(&mut self, data: &[i16]) {
        let mut odata = data.to_vec();
        let mut tail = self.data.split_off(self.selection.0);
        self.data.append(&mut odata);
        self.data.append(&mut tail);
    }

    fn pad_data(&mut self, n: usize) {
        let mut odata = vec![0; n];
        let mut tail = self.data.split_off(self.selection.0);
        self.data.append(&mut odata);
        self.data.append(&mut tail);
    }

    fn switch_mode(&mut self, save: bool) {
        self.edit_mode = !self.edit_mode;
        if self.edit_mode {
            self.edit_buffer = vec![None; 1 + self.selection.1 - self.selection.0];
            self.affected_data = (self.selection.0..=self.selection.1)
                .map(|i| self.data[i])
                .collect();
            self.calc_data(self.selection.0, self.selection.1, false);
        } else {
            if save {
                for (i, s) in self
                    .affected_data
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i + self.selection.0, *s))
                {
                    self.data[i] = s;
                }
            }
            self.affected_data = Vec::new();
            self.edit_buffer = Vec::new();
        }
    }
}

// view + control functions
impl WaveformPage {
    fn side_menu(&self) -> Element<MesDummies> {
        let pdd = 5;
        let nav_menu = self.nav_menu();
        let edit_menu = self.edit_menu();
        let formula_editor = self.parser.element();
        let menu = column![nav_menu, edit_menu, formula_editor]
            .spacing(pdd)
            .padding(pdd)
            .width(Length::Fixed(320.0));
        menu.into()
    }

    fn nav_menu(&self) -> Element<MesDummies> {
        let allign_select = MesDummies::WavePageSig {
            wp_sig: WavePageSig::AllignSelect,
        };
        let fix_neg_scale = MesDummies::WavePageSig {
            wp_sig: WavePageSig::FixNegScale,
        };
        let toggle_edit = |save: bool| MesDummies::WavePageSig {
            wp_sig: WavePageSig::ToggleEditMode { save },
        };
        let reset_view = MesDummies::WavePageSig {
            wp_sig: WavePageSig::ResetView,
        };
        let pdd = 5;
        let but_reset_view = button("Reset View").padding(pdd).on_press(reset_view);
        let but_allign = button("Allign to select")
            .padding(pdd)
            .on_press_maybe((self.selection.0 != self.selection.1).then_some(allign_select));
        let but_fix_negative = button("Flip negative scale")
            .padding(pdd)
            .on_press(fix_neg_scale);
        let but_toggle_edit: Element<MesDummies> = if !self.edit_mode {
            button("Edit Mode")
                .padding(pdd)
                .on_press_maybe(
                    (self.selection.0 != self.selection.1).then_some(toggle_edit(false)),
                )
                .into()
        } else {
            let edit_yes = button("Apply Edit").padding(pdd).on_press_maybe(
                (self.edit_buffer.iter().any(|x| x.is_some())).then_some(toggle_edit(true)),
            );
            let edit_no = button("Discard Edit")
                .padding(pdd)
                .on_press(toggle_edit(false));
            row![edit_yes, edit_no].spacing(pdd).into()
        };
        let menu = column![
            but_reset_view,
            but_allign,
            but_fix_negative,
            but_toggle_edit,
        ]
        .spacing(pdd)
        .padding(pdd)
        // .width(Length::Fixed(320.0))
        ;
        menu.into()
    }

    fn edit_menu(&self) -> Element<MesDummies> {
        let pdd = 5;
        let selecc =
            self.selection.1.max(self.selection.0) - self.selection.0.min(self.selection.1);
        let but_insert = button("Insert blank")
            .padding(pdd)
            .on_press(MesDummies::WavePageSig {
                wp_sig: {
                    WavePageSig::Paste {
                        empty: Some(if selecc == 0 { 16 } else { selecc }),
                    }
                },
            });
        let but_delete = button("Delete")
            .padding(pdd)
            .on_press_maybe((selecc > 0).then_some(MesDummies::WavePageSig {
                wp_sig: WavePageSig::Cut { delete: true },
            }));
        let but_cut = button("Cut")
            .padding(pdd)
            .on_press_maybe((selecc > 0).then_some(MesDummies::WavePageSig {
                wp_sig: WavePageSig::Cut { delete: false },
            }));
        let but_copy = button("Copy")
            .padding(pdd)
            .on_press_maybe((selecc > 0).then_some(MesDummies::WavePageSig {
                wp_sig: WavePageSig::Copy,
            }));
        let but_paste = button("Paste")
            .padding(pdd)
            .on_press(MesDummies::WavePageSig {
                wp_sig: WavePageSig::Paste { empty: None },
            });
        let menu = column![
            but_insert,
            but_delete,
            but_cut,
            but_copy,
            but_paste
        ]
        .spacing(pdd)
        .padding(pdd)
        // .width(Length::Fixed(320.0))
        ;
        menu.into()
    }

    pub fn view(&self) -> Element<'_, MesDummies> {
        let elem = Canvas::new(WaveformDrawer::new(self))
            .width(Length::Fill)
            .height(Length::Fill);
        let rez = row![self.side_menu(), vertical_rule(5), elem];
        rez.into()
    }

    pub fn play_audio(&self, edited: bool) {
        if edited {
            self.play_audio_selected()
        } else {
            self.play_audio_og()
        }
    }

    fn play_audio_og(&self) {
        play_i16_audio(&self.data, self.sample_rate, self.channels);
    }
    #[allow(unused)]
    fn play_audio_edited(&self) {
        let data = [
            self.data[..self.selection.0].to_vec(),
            self.affected_data.clone(),
            self.data[self.selection.1..].to_vec(),
        ]
        .concat();
        // let mut data = self.data.clone();
        // for (i, s) in self
        //     .affected_data
        //     .iter()
        //     .enumerate()
        //     .map(|(i, s)| (i + self.selection.0, *s))
        // {
        //     data[i] = s;
        // }

        play_i16_audio(&data, self.sample_rate, self.channels);
    }
    fn play_audio_selected(&self) {
        if self.edit_mode {
            play_i16_audio(&self.affected_data, self.sample_rate, self.channels);
        } else {
            play_i16_audio(
                &self.data[self.selection.0..self.selection.1],
                self.sample_rate,
                self.channels,
            );
        }
    }

    pub fn process_wave_drawer_sig(&mut self, signal: WaveDrawerSig) {
        use WaveDrawerSig::*;
        match signal {
            Scroll { delta } => {
                self.scroll(delta);
                self.request_redraw();
            }
            ResizeBegin { .. } => {
                if self.edit_mode {
                    self.edit_last_pos = None;
                }
            }
            ResizeOrEraseEnd { .. } => {
                if self.edit_mode {
                    self.edit_last_pos = None;
                }
            }

            ResizeOrErase { scale, mid } => {
                if self.edit_mode {
                    self.erase(mid);
                } else {
                    self.scale(scale);
                }
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

    pub fn process_page_signal(&mut self, signal: WavePageSig, buffer: &mut Vec<i16>) {
        use WavePageSig::*;
        match signal {
            AllignSelect => self.transform.allign_select(self.selection),
            FixNegScale => self.transform.fix_negative(),
            ToggleEditMode { save } => self.switch_mode(save),
            ResetView => self.transform = Transform::default(),
            FormulaChanged(act) => {
                if self.parser.act(act) {
                    self.calc_data(self.selection.0, self.selection.1, false)
                }
            }
            Cut { delete } => {
                let x = self.cut_data();
                if !delete {
                    *buffer = x;
                }
            }
            Copy => *buffer = self.copy_data(),
            Paste { empty } => {
                if let Some(n) = empty {
                    self.pad_data(n);
                } else {
                    self.insert_data(buffer);
                }
            }
        };
        self.request_redraw();
    }

    fn request_redraw(&mut self) {
        self.cache.clear();
    }
}
