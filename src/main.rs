// pub mod spectrum_page;
pub mod waveform_page;

mod data_loader;
mod wav_writer;

mod audio_player;
pub mod not_retarded_vector;

use data_loader::find_file;
use iced::widget::Row;
use itertools::Itertools;
use waveform_page::drawer::WaveDrawerSig;
use waveform_page::sources::{DataSource as _, DataSourceType};
use waveform_page::WavePageSig;

use iced::widget::horizontal_rule;
#[allow(unused_imports)]
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Adio::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct Adio {
    pages: Vec<DataSourceType>,
    cur_page: usize,
    buffer: Vec<i16>,
    sr_n_cn: Option<(u32, u16)>,
}

#[derive(Debug, Clone)]
pub enum MesDummies {
    Fatten,
    SwitchPage(usize),
    OpenFile,
    WriteWav,
    PlayAudio(bool),
    WaveDrawerSig { wd_sig: WaveDrawerSig },
    WavePageSig { wp_sig: WavePageSig },
    AddPage(),
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

    fn page_access(&self) -> Row<'a, MesDummies> {
        let mut buttons: Vec<Element<MesDummies>> = self
            .pages
            .iter()
            .enumerate()
            .map(|(i, _x)| {
                let bname = text(format!(" {:2}", i));
                let b = button(bname)
                    .padding(5)
                    .on_press_maybe((self.cur_page != i).then_some(MesDummies::SwitchPage(i)));
                b.into()
            })
            .collect_vec();
        buttons.insert(
            self.cur_page + 1,
            button(" + ")
                .padding(5)
                .on_press(MesDummies::AddPage())
                .into(),
        );
        let menu: Row<'_, MesDummies> = Row::from_vec(buttons)
            .spacing(5)
            .padding(5)
            .height(Length::Shrink);
        menu
    }

    fn get_cache_for_page(&mut self, page: usize) {
        if page == 0 {
            return;
        }
        match self.cur_page.cmp(&page) {
            std::cmp::Ordering::Less => (),
            std::cmp::Ordering::Equal => return,
            std::cmp::Ordering::Greater => self.pages[self.cur_page].clear_cache(),
        }
        let pos = self
            .pages
            .iter()
            .take(page)
            .rev()
            .position(|x| x.is_cached())
            .expect("first page is a source");
        println!("chaining from {pos}");
        for next in pos + 1..=page {
            let prev = next - 1;
            let data = self.pages[prev].data().unwrap();
            self.pages[prev].clear_cache();
            self.pages[next].set_cache(data);
        }
    }

    // fn get_cache_for_page2(&mut self,page:usize)-> Vec<i16>{
    //     if !self.pages[page].is_cached(){
    //         self.pages[page].set_cache(self.get_cache_for_page2(page-1));
    //     }
    //     let tmp = self.pages[page].data().unwrap();
    //     self.pages[page].clear_cache();
    //     tmp
    // }
}

impl Sandbox for Adio {
    type Message = MesDummies;

    fn new() -> Self {
        let pages = vec![
            DataSourceType::default(),
            DataSourceType::new_wav_page(None),
        ];
        let mut rez = Adio {
            pages,
            cur_page: 0,
            buffer: Vec::new(),
            sr_n_cn: None,
        };
        rez.get_cache_for_page(1);
        rez
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
                self.pages[self.cur_page].process_page_signal(wp_sig, &mut self.buffer)
            }
            MesDummies::WaveDrawerSig { wd_sig } => {
                self.pages[self.cur_page].process_wave_drawer_sig(wd_sig)
            }
            MesDummies::OpenFile => {
                let (data, sample_rate, channels, name) = find_file();
                if data.is_empty() {
                    return;
                }
                self.sr_n_cn = Some((sample_rate, channels));
                let thing1 = DataSourceType::new_widh_data(data, sample_rate, channels, name);
                let thing2 = DataSourceType::new_wav_page(self.sr_n_cn);
                self.pages = vec![thing1, thing2];
                self.cur_page = 0;
            }
            MesDummies::PlayAudio(edited) => {
                self.pages[if !edited {
                    self.cur_page
                } else {
                    self.cur_page.saturating_sub(1)
                }]
                .play_audio();
            }
            MesDummies::WriteWav => {
                self.pages[self.cur_page].save_wav();
            }
            MesDummies::SwitchPage(i) => {
                self.get_cache_for_page(i);
                self.cur_page = i;
            }
            MesDummies::AddPage() => {
                self.pages.insert(
                    self.cur_page + 1,
                    DataSourceType::new_wav_page(self.sr_n_cn),
                );
                self.get_cache_for_page(self.cur_page + 1);
                self.cur_page += 1;
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // let menu: iced::widget::Row<'_, MesDummies> = row![button("Import").padding(5),button("Die").padding(5)].spacing(5).height(30);
        let content = column![
            Adio::top_menu(),
            horizontal_rule(5),
            self.page_access(),
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
