#![allow(unused_imports)]
pub mod drawer;
use iced::advanced::graphics::color;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse::{self, ScrollDelta};
use iced::widget::canvas::Cache;
use iced::widget::Canvas;
use iced::{Element, Length, Rectangle, Renderer, Size, Theme}; //, Vector, Point};

use rand::Rng;

use crate::not_retarded_vector::NRVec;
use crate::MesDummies;

use self::drawer::{Transform, WaveformDrawer};

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
    // selection:(usize,usize),
    transform: Transform,
    cache: Cache,
}

// impl Default for WaveformPage {
//     fn default() -> Self {
//         WaveformPage {
//             data: Vec::default(),
//             transform: Transform::default(),
//             cache: Cache::default(),
//         }
//     }
// }

impl WaveformPage {
    pub fn new_noisy(len: usize) -> Self {
        let mut rng = rand::thread_rng();
        let data = (0..len).map(|_| wrap(rng.gen::<Audi>())).collect();
        WaveformPage {
            data,
            transform: Transform::default(),
            cache: Cache::new(),
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
        }
    }

    pub fn new_widh_data(data: Vec<i16>) -> Self {
        WaveformPage {
            data,
            transform: Transform::default(),
            cache: Cache::new(),
        }
    }

    pub fn append_noise(&mut self, len: usize) {
        let mut rng = rand::thread_rng();
        self.data.extend((0..len).map(|_| wrap(rng.gen::<Audi>())))
    }
}

impl WaveformPage {
    pub fn scroll(&mut self, delta: ScrollDelta) {
        self.transform.scroll(delta);
    }

    pub fn scale(&mut self, scale: NRVec) {
        self.transform.scale(scale);
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }

    fn drawer(&self) -> WaveformDrawer {
        WaveformDrawer::new(self)
    }

    // pub fn view<'a>(&'a self) -> Element<'a, MesDummies> {
    pub fn view(&self) -> Element<'_, MesDummies> {
        Canvas::new(self.drawer())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
