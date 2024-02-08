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

use super::WaveformPage;

use not_retarded_vector::NRVec;

#[derive(Debug)]
pub struct Transform {
    // pos: NRVec,
    scale: NRVec,
    middle_idx: usize,
}

impl Transform {
    pub fn scroll(&mut self, delta: ScrollDelta) {
        let _dy = match delta {
            ScrollDelta::Lines { y, .. } => y,
            ScrollDelta::Pixels { y, .. } => y,
        };
        // self.pos.x += _dy * 8.0;
        // self.middle_idx+=(_dy*8.0).;
    }

    pub fn scale(&mut self, _scale: NRVec) {
        self.scale.x *= _scale.x;
        self.scale.y *= _scale.y;
    }

    /// from graph space to canvas space
    pub fn position_to_canvas(&self, point: NRVec, _bounds: Rectangle) -> NRVec {
        // println!("{:?}",bounds.position());
        let (mut x, mut y) = (
            point.x, // + _bounds.center_x() - _bounds.x,
            point.y, // + _bounds.center_y() - _bounds.y,
        );
        // x += self.pos.x;
        // y += self.pos.y;
        x *= self.scale.x;
        y *= self.scale.y;
        nr_vec(x, y)
    }

    /// from canvas space to graph space
    pub fn canvas_to_position(&self, point: NRVec, _bounds: Rectangle) -> NRVec {
        let (mut x, mut y) = (
            point.x, // - _bounds.center_x() + _bounds.x,
            point.y, // - _bounds.center_y() + _bounds.y,
        );
        x /= self.scale.x;
        y /= self.scale.y;
        // x -= self.pos.x;
        // y -= self.pos.y;
        nr_vec(x, y)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            // pos: nr_vec(0.0, 0.0),
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

    /// from graph space to canvas space
    pub fn position_to_canvas(&self, point: NRVec, bounds: Rectangle) -> NRVec {
        self.parent.transform.position_to_canvas(point, bounds)
    }

    /// from canvas space to graph space
    pub fn canvas_to_position(&self, point: NRVec, bounds: Rectangle) -> NRVec {
        self.parent.transform.canvas_to_position(point, bounds)
    }

    fn get_point(&self, pos: usize, bounds: Rectangle) -> NRVec {
        let x_1: i16 = pos.try_into().expect("ints convert");
        let x: f32 = x_1.try_into().expect("floats convert");
        let y_1: i16 = self.parent.data[pos]; //.try_into().expect("ints convert");
        let y: f32 = y_1.try_into().expect("floats convert");
        let x = x.round();
        let y = y.round();
        self.position_to_canvas(nr_vec(x, y), bounds)
        // Point::new(x, y)
    }

    fn get_point_2(&self, pos: usize, bounds: Rectangle) -> Option<NRVec> {
        let off = self.parent.transform.middle_idx.min(pos);
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
        }; //offset_index
        let i16_x: i16 = i64_x.try_into().ok()?;
        let f32_x: f32 = i16_x.try_into().ok()?;
        let scaled_x = f32_x / self.parent.transform.scale.x;
        None
    }

    fn path(&self, bounds: Rectangle) -> Path {
        let mut res = Builder::new();
        for pos in 0..self.parent.data.len() {
            res.line_to(self.get_point(pos, bounds).into());
        }
        res.build()
    }
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
                    WDStates::Resizing { one } => {
                        (Status::Captured, Some(MesDummies::ResizeEnd { end: *one }))
                    }
                    WDStates::Selecting => (
                        Status::Captured,
                        Some(MesDummies::SelectEnd {
                            end: nr_vec(0.0, 0.0),
                        }),
                    ),
                    WDStates::Idle => (Status::Ignored, None),
                };
                *_state = WDStates::Idle;
                return message;
            }
        };
        match _event {
            CanEvent::Mouse(mouse_event) => {
                let message = match mouse_event {
                    mouse::Event::CursorMoved { position } => match _state {
                        WDStates::Resizing { one } => {
                            let cursor = NRVec::from(position) - _bounds.center();
                            let scale_x = cursor.x / one.x;
                            let scale_y = cursor.y / one.y;

                            *_state = WDStates::Resizing { one: cursor };
                            Some(MesDummies::Resize {
                                scale: NRVec {
                                    x: scale_x,
                                    y: scale_y,
                                },
                            })
                        }
                        WDStates::Idle => Some(MesDummies::ForceRedraw),
                        _ => None,
                    },
                    mouse::Event::ButtonPressed(mouse_button) => match _state {
                        WDStates::Idle => match mouse_button {
                            mouse::Button::Right => {
                                let cursor = NRVec::from(cursor_position) - _bounds.center()
                                    + _bounds.position();
                                let begin = cursor;
                                // self.canvas_to_position(cursor, _bounds);
                                *_state = WDStates::Resizing { one: begin };
                                Some(MesDummies::ResizeBegin { begin })
                            }
                            mouse::Button::Left => {
                                *_state = WDStates::Selecting;
                                let mut begin: NRVec = cursor_position.into();
                                begin.x /= _bounds.width;
                                begin.y /= _bounds.height;
                                Some(MesDummies::SelectBegin { begin })
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    mouse::Event::ButtonReleased(mouse_button) => match _state {
                        WDStates::Resizing { .. } => {
                            if let mouse::Button::Right = mouse_button {
                                let end = self.canvas_to_position(cursor_position.into(), _bounds);
                                *_state = WDStates::Idle;
                                Some(MesDummies::ResizeEnd { end })
                            } else {
                                None
                            }
                        }
                        WDStates::Selecting => {
                            if let mouse::Button::Left = mouse_button {
                                *_state = WDStates::Idle;
                                let mut end: NRVec = cursor_position.into();
                                end.x /= _bounds.width;
                                end.y /= _bounds.height;
                                Some(MesDummies::SelectEnd { end })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    mouse::Event::WheelScrolled { delta } => Some(MesDummies::Scroll { delta }),
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

        let cur_pos: NRVec = NRVec::from(_cursor.position_in(bounds).unwrap_or_default())
            - bounds.center()
            + bounds.position();

        let stroke = Stroke::default().with_color(color).with_width(1.0);
        let grid_style = Stroke::default().with_color(Color::from_rgba8(200, 200, 200, 0.75));

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
            frame.stroke(
                &Path::line(nr_vec(0.0, 0.0).into(), nr_vec(w, h).into()),
                grid_style.clone().with_width(1.5),
            );
            frame.stroke(
                &Path::line(nr_vec(w, 0.0).into(), nr_vec(0.0, h).into()),
                grid_style.clone().with_width(1.5),
            );

            let translation: NRVec = frame.center().into();
            frame.translate(translation.into());

            // let cur_pos = _cursor.position_in(bounds).unwrap_or_default()-frame.center();
            // let cur_pos = Point::new(cur_pos.x, cur_pos.y);
            // let cur_pos = self.canvas_to_position(cur_pos, bounds);
            // let cur_pos = self.position_to_canvas(cur_pos, bounds);
            frame.stroke(
                &Path::line(nr_vec(0.0, 0.0).into(), cur_pos.into()),
                grid_style
                    .clone()
                    .with_color(Color::from_rgb8(100, 255, 200))
                    .with_width(2.0),
            );

            if let WDStates::Resizing { one } = _state {
                let pos = *one; //self.position_to_canvas(*one, bounds);
                frame.stroke(
                    &Path::line(nr_vec(0.0, 0.0).into(), pos.into()),
                    grid_style.clone().with_width(2.0),
                );
            }

            // if !true {
            //     let pos = frame.center();
            //     frame.stroke(
            //         &Path::line(Point::new(w, h), pos),
            //         grid_style.clone().with_width(2.0),
            //     );
            // }
            frame.stroke(&self.path(bounds), stroke);
        });
        vec![content]
    }
}
