use iced::widget::Canvas;
use iced::widget::{canvas::Cache, text_input, vertical_rule};
use iced::{Element, Length};
use vector2d::Vector2D;

use iced::widget::{column, row};

use crate::MesDummies;

use self::drawer::SpectrumDrawer;
pub mod drawer;

type Spec = i16;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Usize2d {
    x: usize,
    y: usize,
}

#[derive(Debug)]
pub struct SpectrumPage {
    data: Vector2D<Spec>,
    selection: (Usize2d, Usize2d),
    edit_last_pos: Option<Usize2d>,
    cache: Cache,
}

impl Default for SpectrumPage {
    fn default() -> Self {
        Self {
            data: Vector2D::new(0, 0),
            selection: Default::default(),
            edit_last_pos: Default::default(),
            cache: Cache::new(),
        }
    }
}

impl SpectrumPage {
    fn side_menu(&self) -> Element<'static, MesDummies> {
        let pdd = 5;
        let formula_editor = text_input("s[0]=m", "s[0]=m").width(256);
        // .on_input(formula_edit);
        let menu = column![formula_editor]
            .spacing(pdd)
            .padding(pdd)
            .width(Length::Shrink);
        menu.into()
    }

    fn request_redraw(&mut self) {
        self.cache.clear();
    }

    fn drawer(&self) -> SpectrumDrawer {
        SpectrumDrawer::new(self)
    }

    pub fn view(&self) -> Element<'_, MesDummies> {
        let elem = Canvas::new(self.drawer())
            .width(Length::Fill)
            .height(Length::Fill);
        let rez = row![self.side_menu(), vertical_rule(5), elem];
        rez.into()
    }
}
