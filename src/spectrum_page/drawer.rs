use iced::widget::canvas::Program;

use crate::{
    not_retarded_vector::{nr_vec, NRVec},
    MesDummies,
};

use super::SpectrumPage;

use iced::{
    advanced::mouse,
    widget::canvas::{Geometry, Path, Stroke},
    Color, Rectangle, Renderer, Theme,
};

pub struct SpectrumDrawer<'w> {
    pub parent: &'w SpectrumPage,
}

impl<'w> SpectrumDrawer<'w> {
    pub fn new(parent: &'w SpectrumPage) -> Self {
        SpectrumDrawer { parent }
    }
}

#[derive(Debug)]
pub enum SPStates {
    Resizing { one: NRVec },
    Selecting,
    Editing,
    Idle,
}

impl Default for SPStates {
    fn default() -> Self {
        Self::Idle
    }
}

impl Program<MesDummies> for SpectrumDrawer<'_> {
    type State = SPStates;

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

            frame.stroke(
                &Path::line(nr_vec(0.0, 0.0).into(), cur_pos.into()),
                grid_style
                    .clone()
                    .with_color(Color::from_rgb8(100, 255, 200))
                    .with_width(2.0),
            );
            // frame.stroke(&self.path(bounds), stroke);

            // let bounds = self.selection_lines(bounds);
            // if let Some(left) = &bounds.0 {
            //     frame.stroke(left, select_style.clone());
            // }
            // if let Some(right) = &bounds.1 {
            //     frame.stroke(right, select_style);
            // }
        });
        vec![content]
    }
}
