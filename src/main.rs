pub mod waveform_page;

mod data_loader;

mod audio_player;
pub mod not_retarded_vector;

use data_loader::find_file;
use iced::widget::Row;
use waveform_page::drawer::WaveDrawerSig;
use waveform_page::WavePageSig;

use crate::waveform_page::WaveformPage;
use iced::widget::horizontal_rule;
#[allow(unused_imports)]
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Sandbox, Settings}; //, Point};

pub fn main() -> iced::Result {
    Adio::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct Adio {
    // hide_audio: bool,
    pages: Vec<WaveformPage>,
    cur_page: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MesDummies {
    Fatten,
    // Scroll { delta: ScrollDelta },
    // ResizeBegin { begin: NRVec },
    // ResizeEnd { end: NRVec },
    // Resize { scale: NRVec },
    // SelectBegin { begin: NRVec },
    // SelectEnd { end: NRVec },
    // Select { mid: NRVec },
    // ForceRedraw,
    OpenFile,
    PlayAudio,
    WaveDrawerSig { wd_sig: WaveDrawerSig },
    WavePageSig { wp_sig: WavePageSig },
}

impl<'a> Adio {
    fn top_menu() -> Row<'a, MesDummies> {
        let menu: Row<'_, MesDummies> = row![
            button("Import").padding(5).on_press(MesDummies::OpenFile),
            button("Play").padding(5).on_press(MesDummies::PlayAudio)
        ]
        .spacing(5)
        .padding(5)
        .height(Length::Shrink);
        menu
    }
}

impl Sandbox for Adio {
    type Message = MesDummies;

    fn new() -> Self {
        let mut res = Adio::default();
        res.pages.push(WaveformPage::new_wedge(64, 32));
        res.pages.push(WaveformPage::new_noisy(128));
        res
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn title(&self) -> String {
        String::from("Adios!")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            MesDummies::Fatten => {
                // self.hide_audio = !self.hide_audio;
                self.cur_page = 1 - self.cur_page;
                // self.pages[self.cur_page].append_noise(16);
            }
            // MesDummies::Scroll { delta } => {
            //     self.pages[self.cur_page].scroll(delta);
            //     self.pages[self.cur_page].request_redraw();
            // }
            // MesDummies::ResizeBegin { begin } => println!("resize begin from {:?}", begin),
            // MesDummies::ResizeEnd { end } => println!("resize  end   at  {:?}", end),

            // MesDummies::Resize { scale } => {
            //     self.pages[self.cur_page].scale(scale);
            //     self.pages[self.cur_page].request_redraw();
            // }

            // MesDummies::SelectBegin { begin } => {
            //     self.pages[self.cur_page].select_begin(begin);
            //     // println!("select begin from {:?}", begin)
            //     self.pages[self.cur_page].request_redraw();
            // }
            // MesDummies::Select { mid } => {
            //     self.pages[self.cur_page].select_end(mid);
            //     self.pages[self.cur_page].request_redraw();
            // }
            // MesDummies::SelectEnd { end } => {
            //     self.pages[self.cur_page].select_end(end);
            //     self.pages[self.cur_page].fix_select();
            //     // println!("select  end   at  {:?}", end);
            //     self.pages[self.cur_page].request_redraw();
            // }

            // MesDummies::ForceRedraw => self.pages[self.cur_page].request_redraw(),
            MesDummies::WavePageSig { wp_sig } => {
                self.pages[self.cur_page].process_page_signal(wp_sig)
            }
            MesDummies::WaveDrawerSig { wd_sig } => {
                self.pages[self.cur_page].process_wave_drawer_sig(wd_sig)
            }
            MesDummies::OpenFile => {
                let (data, sample_rate, channels) = find_file();
                if data.is_empty() {
                    return;
                }
                self.pages[self.cur_page] =
                    WaveformPage::new_widh_data(data, sample_rate, channels);
            }
            MesDummies::PlayAudio => {
                self.pages[self.cur_page].play_audio();
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // let menu: iced::widget::Row<'_, MesDummies> = row![button("Import").padding(5),button("Die").padding(5)].spacing(5).height(30);
        let content = column![
            Adio::top_menu(),
            horizontal_rule(5),
            // button("Yay")
            // .padding(40)
            // .width(Length::Fill)
            // .height(Length::Fill),
            self.pages[self.cur_page].view()
        ]
        .align_items(Alignment::Start);

        // let widgett = self.pages[self.cur_page].view();
        // let hid_wid = text("hidden");
        // let widgett2 = button("Die").padding(40).on_press(MesDummies::Fatten);
        // let content = if !self.hide_audio {
        //     row![widgett2, widgett]
        // } else {
        //     row![hid_wid, widgett2]
        // }
        // .padding(20)
        // .spacing(20)
        // .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
