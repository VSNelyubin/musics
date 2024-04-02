// pub mod spectrum_page;
pub mod waveform_page;

mod data_loader;
mod wav_writer;

mod audio_player;
pub mod not_retarded_vector;

use data_loader::find_file;
use iced::widget::Row;
// use spectrum_page::SpectrumPage;
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

pub enum Pages {
    Wave(WaveformPage),
    // Spec(SpectrumPage),
    OOga,
}

#[allow(dead_code)]
impl Pages {
    fn new_wedge(len: usize, focus: i16) -> Self {
        Self::Wave(WaveformPage::new_wedge(len, focus))
    }
    fn new_noisy(len: usize) -> Self {
        Self::Wave(WaveformPage::new_noisy(len))
    }
    fn new_widh_data(data: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        Self::Wave(WaveformPage::new_widh_data(data, sample_rate, channels))
    }
    fn save_wav(&self) {
        if let Self::Wave(wave) = self {
            wave.save_wav()
        } else {
            panic!()
        }
    }
    fn process_page_signal(&mut self, signal: WavePageSig) {
        if let Self::Wave(wave) = self {
            wave.process_page_signal(signal)
        } else {
            panic!()
        }
    }
    fn process_wave_drawer_sig(&mut self, signal: WaveDrawerSig) {
        if let Self::Wave(wave) = self {
            wave.process_wave_drawer_sig(signal)
        } else {
            panic!()
        }
    }
    fn play_audio(&self, edited: bool) {
        if let Self::Wave(wave) = self {
            wave.play_audio(edited)
        } else {
            panic!()
        }
    }
    fn view(&self) -> Element<'_, MesDummies> {
        match self {
            Self::Wave(wave) => wave.view(),
            // Self::Spec(spec) => spec.view(),
            _ => panic!(),
        }
    }
}

#[derive(Default)]
struct Adio {
    // hide_audio: bool,
    pages: Vec<Pages>,
    cur_page: usize,
}

#[derive(Debug, Clone)]
pub enum MesDummies {
    Fatten,
    OpenFile,
    WriteWav,
    PlayAudio(bool),
    WaveDrawerSig { wd_sig: WaveDrawerSig },
    WavePageSig { wp_sig: WavePageSig },
}

impl<'a> Adio {
    fn top_menu() -> Row<'a, MesDummies> {
        let menu: Row<'_, MesDummies> = row![
            button("Import").padding(5).on_press(MesDummies::OpenFile),
            button("Play")
                .padding(5)
                .on_press(MesDummies::PlayAudio(false)),
            button("Play Edited")
                .padding(5)
                .on_press(MesDummies::PlayAudio(true)),
            button("Save Wav").padding(5).on_press(MesDummies::WriteWav) // button("Flip Page").padding(5).on_press(MesDummies::Fatten)
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
        res.pages.push(Pages::new_wedge(64, 32));
        // res.pages.push(Pages::new_noisy(128));
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
                self.pages[self.cur_page] = Pages::new_widh_data(data, sample_rate, channels);
            }
            MesDummies::PlayAudio(edited) => {
                self.pages[self.cur_page].play_audio(edited);
            }
            MesDummies::WriteWav => {
                self.pages[self.cur_page].save_wav();
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

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
