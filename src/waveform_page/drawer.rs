use audrey::hound::Sample;
use iced::futures::future::select;
use iced::mouse::ScrollDelta;
use iced::widget::canvas::event::{self, Event as CanEvent};
use iced::{
    advanced::mouse,
    event::Status,
    gradient::{self, Linear},
    widget::canvas::{
        path::{lyon_path::math::Translation, Builder},
        Geometry, Path, Program, Stroke,
    },
    Background, Color, Event, Rectangle, Renderer, Theme,
};
use tracing_subscriber::field::display::Messages;

use crate::not_retarded_vector::nr_vec;
use crate::{not_retarded_vector, MesDummies};

use super::{Audi, WaveformPage};

use not_retarded_vector::NRVec;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Transform {
    pos: NRVec,
    scale: NRVec,
    middle_idx: usize,
}

impl Transform {
    pub fn new(scale: NRVec, off: usize) -> Self {
        Self {
            pos: NRVec::default(),
            scale,
            middle_idx: off,
        }
    }

    pub fn scroll(&mut self, delta: ScrollDelta, max: usize) {
        let _dy = match delta {
            ScrollDelta::Lines { y, .. } => y,
            ScrollDelta::Pixels { y, .. } => y,
        };
        // self.pos.x += _dy * 8.0 / (1.0 + self.scale.x);
        // let delt = (_dy * 8.0 / (1.0 + self.scale.x)) as i64;
        // if delt < 0 {
        //     let abs: usize = delt.abs().try_into().unwrap();
        //     self.middle_idx -= abs.min(self.middle_idx);
        // } else {
        //     let abs: usize = delt.abs().try_into().unwrap();
        //     self.middle_idx += abs;
        // }
        let delt = _dy * 8.0 / self.scale.x;
        // print!("{:10.4} ", delt);
        let delt = (delt as i64).abs();
        // println!(" {delt}");
        self.middle_idx = if _dy < 0.0 {
            self.middle_idx.saturating_sub(1 + delt as usize)
        } else {
            self.middle_idx.saturating_add(1 + delt as usize).min(max)
        };
        // println!("{}\t{}", self.pos.x, self.middle_idx);
    }

    pub fn scale(&mut self, _scale: NRVec) {
        self.scale.x *= _scale.x;
        self.scale.y *= _scale.y;
        // println!("{}",(-(self.scale.x.abs().log2()).floor() as usize).next_power_of_two());
    }

    pub fn get_pos(&self, pos: f32) -> usize {
        let scaled = pos / self.scale.x;
        let scaled = scaled + scaled.signum() / 2.0;
        let scaled = scaled as i64;
        if scaled < 0 {
            self.middle_idx.saturating_sub((-scaled) as usize)
        } else {
            self.middle_idx
                .checked_add(scaled as usize)
                .unwrap_or(usize::MAX)
        }
    }

    pub fn get_amp(&self, high: f32) -> Audi {
        let scaled = high / self.scale.y;
        let scaled = (scaled + 0.5) as i64;
        scaled
            .try_into()
            .unwrap_or(if high > 0.0 { Audi::MAX } else { Audi::MIN })
    }

    pub fn allign_select(&mut self, selection: (usize, usize)) {
        let delt = selection.0.max(selection.1) - selection.0.min(selection.1);
        self.middle_idx = selection.0.min(selection.1) + delt / 2;
        let scale: i64 = delt.try_into().unwrap();
        let scale: f64 = scale as f64;
        let scale = 700.0 / (scale + 0.1);
        let scale: f32 = scale as f32;
        self.scale.x = scale * self.scale.x.signum();
    }

    pub fn fix_negative(&mut self) {
        self.scale = NRVec {
            x: self.scale.x.abs(),
            y: self.scale.y.abs(),
        };
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            pos: nr_vec(0.0, 0.0),
            scale: nr_vec(1.0, 1.0),
            middle_idx: 0,
        }
    }
}

pub struct WaveformDrawer<'w> {
    pub parent: &'w WaveformPage,
}

impl<'w> WaveformDrawer<'w> {
    pub fn new(parent: &'w WaveformPage) -> Self {
        WaveformDrawer { parent }
    }

    fn get_point_y(&self, pos: usize) -> f32 {
        let y_1: i16 = self.parent.data[pos]; //.try_into().expect("ints convert");
        let y: f32 = y_1.into(); //try_into().expect("floats convert");
        y * self.parent.transform.scale.y
    }

    fn get_point_by(&self, pos: usize) -> f32 {
        let y_1: i16 = self.parent.affected_data[pos - self.parent.selection.0];
        let y: f32 = y_1.into();
        y * self.parent.transform.scale.y
    }

    fn get_point_x(&self, pos: usize, bounds: Rectangle) -> Option<f32> {
        let off = self.parent.transform.middle_idx; //.min(pos);
        let i64_x: i64 = if let (Ok(s_pos), Ok(s_off)) = (pos.try_into(), off.try_into()) {
            let poss: i64 = s_pos;
            let offs: i64 = s_off;
            poss - offs
        } else {
            let rez = (pos - off).try_into();
            if let Ok(r) = rez {
                r
            } else {
                return None;
            }
        };
        let scaled_x = if i64_x.abs() >= 0x8000 {
            let scale = (1. / self.parent.transform.scale.x) as i64;
            (scale != 0).then_some(0)?;
            let i16_x: i16 = (i64_x / scale).try_into().ok()?;
            i16_x.into()
        } else {
            let i16_x: i16 = i64_x.try_into().ok()?;
            let f32_x: f32 = i16_x.into();
            f32_x * self.parent.transform.scale.x
        };
        let point = nr_vec(scaled_x, 0.0);
        let ofpoint = point - nr_vec(0.0, point.y) + bounds.center();
        bounds.contains(ofpoint.into()).then_some(scaled_x)
    }

    #[allow(unused)]
    fn get_point_2(&self, pos: usize, bounds: Rectangle, use_buf: bool) -> Option<NRVec> {
        let scaled_x = self.get_point_x(pos, bounds)?;
        let scaled_y = if use_buf {
            self.get_point_by(pos)
        } else {
            self.get_point_y(pos)
        };

        let point = nr_vec(scaled_x, scaled_y);
        let ofpoint = point - nr_vec(0.0, point.y) + bounds.center();
        Some(point)
        // bounds.contains(ofpoint.into()).then_some(point)
    }

    fn selection_lines(&self, bounds: Rectangle) -> (Option<Path>, Option<Path>) {
        let x2p = |x: f32| {
            let bot = nr_vec(x, -bounds.height / 2.0);
            let top = nr_vec(x, bounds.height / 2.0);
            Path::line(bot.into(), top.into())
        };
        (
            self.get_point_x(self.parent.selection.0, bounds).map(x2p),
            self.get_point_x(self.parent.selection.1, bounds).map(x2p),
        )
    }

    fn limit_lines(&self, bounds: Rectangle) -> Option<(Path, Path)> {
        let x2p = |y: f32| {
            let left = nr_vec(-bounds.width / 2.0, y);
            let right = nr_vec(bounds.width / 2.0, y);
            Path::line(left.into(), right.into())
        };
        let y_1 = i16::MAX;
        let y_2: f32 = y_1.into();
        let y_3 = y_2 * self.parent.transform.scale.y;
        // bounds
        //     .contains(nr_vec(0., y_3).into())
        //     .then_some()
        Some((x2p(y_3), x2p(-y_3)))
    }

    fn time_marks(&self, bounds: Rectangle) -> Vec<Path> {
        let x2p = |x: f32, h: f32| {
            let left = nr_vec(x, -h);
            let right = nr_vec(x, h);
            Path::line(left.into(), right.into())
        };

        let rez: Vec<_> = (0..self.parent.data.len())
            .step_by(self.parent.sample_rate as usize * self.parent.channels as usize)
            .map(|x| self.get_point_x(x, bounds))
            .skip_while(|x| x.is_none())
            .map_while(|x| x.map(|y| x2p(y, 50.)))
            .collect();
        // println!("{}",rez.len());
        assert!(rez.len() as isize / 2 >= 0);
        rez
    }

    fn iter_step(&self) -> usize {
        ((0.1 / self.parent.transform.scale.x.abs()).floor() as usize).max(1)
    }

    fn path(&self, bounds: Rectangle) -> Path {
        let mut res = Builder::new();
        for pnt in (0..self.parent.data.len())
            .step_by(self.iter_step())
            .map(|pos| self.get_point_2(pos, bounds, false))
            .skip_while(|x| x.is_none())
            .map_while(|x| x)
        {
            res.line_to(pnt.into());
        }
        res.build()
    }

    fn unselected_paths(&self, bounds: Rectangle) -> (Path, Path) {
        let mut left = Builder::new();
        for pnt in (0..=self.parent.selection.0)
            .step_by(self.iter_step())
            .map(|pos| self.get_point_2(pos, bounds, false))
            .skip_while(|x| x.is_none())
            .map_while(|x| x)
        {
            left.line_to(pnt.into());
        }
        let mut right = Builder::new();
        for pnt in (self.parent.selection.1..self.parent.data.len())
            .step_by(self.iter_step())
            .map(|pos| self.get_point_2(pos, bounds, false))
            .skip_while(|x| x.is_none())
            .map_while(|x| x)
        {
            right.line_to(pnt.into());
        }
        (left.build(), right.build())
    }

    fn edit_path(&self, bounds: Rectangle) -> Path {
        let mut res = Builder::new();
        for pnt in (self.parent.selection.0..=self.parent.selection.1)
            .step_by(self.iter_step())
            .map(|pos| self.get_point_2(pos, bounds, true))
            .skip_while(|x| x.is_none())
            .map_while(|x| x)
        {
            res.line_to(pnt.into());
        }
        res.build()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WaveDrawerSig {
    Scroll { delta: ScrollDelta },
    ResizeBegin { begin: NRVec },
    ResizeOrEraseEnd { end: NRVec },
    ResizeOrErase { scale: NRVec, mid: NRVec },
    SelectOrEditBegin { begin: NRVec },
    SelectOrEditEnd { end: NRVec },
    SelectOrEdit { mid: NRVec },
    ForceRedraw,
}

#[derive(Debug)]
pub enum WDStates {
    Resizing { one: NRVec },
    Selecting,
    Idle,
}

impl Default for WDStates {
    fn default() -> Self {
        Self::Idle
    }
}

impl Program<MesDummies> for WaveformDrawer<'_> {
    type State = WDStates;

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: CanEvent,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<MesDummies>) {
        // println!("state = {:?}", _state);
        let cursor_position = match _cursor.position_in(_bounds) {
            Some(pos) => pos,
            None => {
                let message = match _state {
                    WDStates::Resizing { one } => (
                        Status::Captured,
                        Some(MesDummies::WaveDrawerSig {
                            wd_sig: WaveDrawerSig::ResizeOrEraseEnd { end: *one },
                        }),
                    ),
                    WDStates::Selecting => (
                        Status::Captured,
                        Some(MesDummies::WaveDrawerSig {
                            wd_sig: WaveDrawerSig::SelectOrEditEnd {
                                end: nr_vec(0.0, 0.0),
                            },
                        }),
                    ),
                    WDStates::Idle => (Status::Ignored, None),
                };
                *_state = WDStates::Idle;
                return message;
            }
        };
        let mut supos: NRVec = cursor_position.into();
        supos.x -= _bounds.width / 2.0;
        supos.y -= _bounds.height / 2.0;
        match _event {
            CanEvent::Mouse(mouse_event) => {
                let message = match mouse_event {
                    mouse::Event::CursorMoved { position } => match _state {
                        WDStates::Resizing { one } => {
                            let cursor = NRVec::from(position) - _bounds.center();
                            let scale_x = cursor.x / one.x;
                            let scale_y = cursor.y / one.y;

                            *_state = WDStates::Resizing { one: cursor };
                            Some(MesDummies::WaveDrawerSig {
                                wd_sig: WaveDrawerSig::ResizeOrErase {
                                    scale: NRVec {
                                        x: scale_x,
                                        y: scale_y,
                                    },
                                    mid: supos,
                                },
                            })
                        }
                        WDStates::Selecting => Some(MesDummies::WaveDrawerSig {
                            wd_sig: WaveDrawerSig::SelectOrEdit { mid: supos },
                        }),
                        WDStates::Idle => Some(MesDummies::WaveDrawerSig {
                            wd_sig: WaveDrawerSig::ForceRedraw,
                        }),
                        // _ => None,
                    },
                    mouse::Event::ButtonPressed(mouse_button) => match _state {
                        WDStates::Idle => match mouse_button {
                            mouse::Button::Right => {
                                *_state = WDStates::Resizing { one: supos };
                                Some(MesDummies::WaveDrawerSig {
                                    wd_sig: WaveDrawerSig::ResizeBegin { begin: supos },
                                })
                            }
                            mouse::Button::Left => {
                                *_state = WDStates::Selecting;
                                Some(MesDummies::WaveDrawerSig {
                                    wd_sig: WaveDrawerSig::SelectOrEditBegin { begin: supos },
                                })
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    mouse::Event::ButtonReleased(mouse_button) => match _state {
                        WDStates::Resizing { .. } => {
                            if let mouse::Button::Right = mouse_button {
                                *_state = WDStates::Idle;
                                Some(MesDummies::WaveDrawerSig {
                                    wd_sig: WaveDrawerSig::ResizeOrEraseEnd { end: supos },
                                })
                            } else {
                                None
                            }
                        }
                        WDStates::Selecting => {
                            if let mouse::Button::Left = mouse_button {
                                *_state = WDStates::Idle;
                                Some(MesDummies::WaveDrawerSig {
                                    wd_sig: WaveDrawerSig::SelectOrEditEnd { end: supos },
                                })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    mouse::Event::WheelScrolled { delta } => match _state {
                        WDStates::Resizing { .. } => None,
                        WDStates::Selecting => Some(MesDummies::WaveDrawerSig {
                            wd_sig: WaveDrawerSig::Scroll { delta },
                        }),
                        WDStates::Idle => Some(MesDummies::WaveDrawerSig {
                            wd_sig: WaveDrawerSig::Scroll { delta },
                        }),
                    },
                    _ => None,
                };
                (event::Status::Captured, message)
            }
            _ => (Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let bg = Color::from_rgb8(10, 35, 50);
        let color = Color::from_rgb8(255, 100, 10);

        // let cur_pos: NRVec = NRVec::from(_cursor.position_in(bounds).unwrap_or_default())
        //     - bounds.center()
        //     + bounds.position();

        let stroke = Stroke::default()
            .with_color(color)
            .with_width(1.0)
            .with_line_join(iced::widget::canvas::LineJoin::Bevel);
        let grid_style = Stroke::default().with_color(Color::from_rgba8(200, 200, 200, 0.75));

        let select_style = Stroke::default().with_color(Color::from_rgba8(100, 50, 200, 0.75));

        let content = self.parent.cache.draw(renderer, bounds.size(), |frame| {
            // let (w, h) = (frame.size().width, frame.size().height);
            // let bounds: Rectangle = Rectangle::with_size(frame.size());
            let (w, h) = (bounds.width, bounds.height);
            frame.fill_rectangle(nr_vec(0.0, 0.0).into(), frame.size(), bg);
            frame.stroke(
                &Path::line(nr_vec(0.0, h * 0.5).into(), nr_vec(w, h * 0.5).into()),
                grid_style.clone().with_width(1.5),
            );
            frame.stroke(
                &Path::line(nr_vec(w * 0.5, 0.0).into(), nr_vec(w * 0.5, h).into()),
                grid_style.clone().with_width(1.5),
            );

            let translation: NRVec = frame.center().into();
            frame.translate(translation.into());

            if self.parent.edit_mode {
                let paths = self.unselected_paths(bounds);
                frame.stroke(&paths.0, stroke.clone());
                frame.stroke(&paths.1, stroke.clone());
                frame.stroke(
                    &self.edit_path(bounds),
                    stroke.with_color(Color::from_rgb8(100, 255, 200)),
                )
            } else {
                frame.stroke(&self.path(bounds), stroke);
            }

            for i in self.time_marks(bounds) {
                frame.stroke(&i, grid_style.clone().with_width(1.1));
            }

            let selecc = self.selection_lines(bounds);
            if let Some(left) = &selecc.0 {
                frame.stroke(left, select_style.clone());
            }
            if let Some(right) = &selecc.1 {
                frame.stroke(right, select_style.clone());
            }
            if let Some((top, bot)) = self.limit_lines(bounds) {
                frame.stroke(&top, select_style.clone());
                frame.stroke(&bot, select_style);
            }
        });
        vec![content]
    }
}
