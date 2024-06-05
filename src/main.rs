// pub mod spectrum_page;
pub mod waveform_page;

mod data_loader;
mod wav_writer;

mod audio_player;
pub mod not_retarded_vector;

use data_loader::find_file;
use iced::{widget::Row, window::settings};
use sonogram::{ColourGradient, ColourTheme, SpecOptionsBuilder};
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
    fn process_page_signal(&mut self, signal: WavePageSig, buffer: &mut Vec<i16>) {
        if let Self::Wave(wave) = self {
            wave.process_page_signal(signal, buffer)
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

const SPEC_LEN: usize = 1 << 11;

#[derive(Default)]
struct Adio {
    // hide_audio: bool,
    pages: Vec<WaveformPage>,
    cur_page: usize,
    buffer: Vec<i16>,
    cached_spec: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum MesDummies {
    GetSpec,
    ClearSpec,
    NewWindow,
    OpenFile,
    WriteWav,
    PlayAudio(bool),
    WaveDrawerSig { wd_sig: WaveDrawerSig },
    WavePageSig { wp_sig: WavePageSig },
}

impl<'a> Adio {
    fn top_menu(&self) -> Row<'a, MesDummies> {
        let menu: Row<'_, MesDummies> = row![
            button("Import").padding(5).on_press(MesDummies::OpenFile),
            button("Play")
                .padding(5)
                .on_press(MesDummies::PlayAudio(false)),
            button("Play Selected")
                .padding(5)
                .on_press(MesDummies::PlayAudio(true)),
            button("Save Wav").padding(5).on_press(MesDummies::WriteWav),
            button("Spectrogram").padding(5).on_press_maybe(
                (self.pages[self.cur_page].select_len() > 1).then_some(MesDummies::GetSpec)
            ),
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
            MesDummies::GetSpec => {
                self.cached_spec = get_spec(
                    self.pages[self.cur_page].focus_data(),
                    self.pages[self.cur_page].sample_rate(),
                )
            }
            MesDummies::ClearSpec => self.cached_spec.clear(),
            MesDummies::NewWindow => {
                let _x: (iced::window::Id, iced::Command<MesDummies>) =
                    iced::window::spawn(settings::Settings::default());
            }
            MesDummies::WavePageSig { wp_sig } => {
                self.pages[self.cur_page].process_page_signal(wp_sig, &mut self.buffer)
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
            self.top_menu(),
            horizontal_rule(5),
            // button("Yay")
            // .padding(40)
            // .width(Length::Fill)
            // .height(Length::Fill),
            iced::widget::image::Image::new(iced::widget::image::Handle::from_memory(
                self.cached_spec.clone()
            ))
            .width(Length::Fill),
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

fn get_spec(waveform: Vec<i16>, sr: u32) -> Vec<u8> {
    // Build the model
    let waveform = if waveform.len() <= SPEC_LEN {
        waveform
            .iter()
            .cycle()
            .take(SPEC_LEN * 16)
            .cloned()
            .collect()
    } else {
        waveform
    };
    let mut spectrograph = SpecOptionsBuilder::new(SPEC_LEN) //.min(waveform.len().next_power_of_two() >> 1))
        .load_data_from_memory(waveform, sr)
        .build()
        .unwrap();

    // Compute the spectrogram giving the number of bins and the window overlap.
    let mut spectrogram = spectrograph.compute();

    // Specify a colour gradient to use (note you can create custom ones)
    let mut gradient = ColourGradient::create(ColourTheme::Default);

    spectrogram
        .to_png_in_memory(sonogram::FrequencyScale::Log, &mut gradient, 1000, 150)
        .unwrap()
}
