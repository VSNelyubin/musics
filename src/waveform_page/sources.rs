use iced::{
    widget::{text, Button},
    Element, Length,
};

use crate::{audio_player::play_i16_audio, wav_writer::save_wav, MesDummies};

use super::{drawer::WaveDrawerSig, wrap, WavePageSig, WaveformPage};

pub struct PCMFileSource {
    data: Vec<i16>,
    sample_rate: u32,
    channels: u16,
    source_name: Option<String>,
}

impl Default for PCMFileSource {
    fn default() -> Self {
        Self {
            data: vec![0; 256],
            sample_rate: 44100,
            channels: 1,
            source_name: None,
        }
    }
}

impl PCMFileSource {
    fn new_widh_data(data: Vec<i16>, sample_rate: u32, channels: u16, name: impl ToString) -> Self {
        Self {
            data,
            sample_rate,
            channels,
            source_name: Some(name.to_string()),
        }
    }

    fn new_noisy(len: usize) -> Self {
        let mut rng = rand::thread_rng();
        let data = (0..len)
            .map(|_| wrap(rand::Rng::gen::<i16>(&mut rng)))
            .collect();
        Self {
            data,
            sample_rate: 44100,
            channels: 1,
            source_name: None,
        }
    }

    fn new_wedge(len: usize, focus: i16) -> Self {
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
        Self {
            data,
            sample_rate: 44100,
            channels: 1,
            source_name: None,
        }
    }

    pub fn view(&self) -> Element<'_, MesDummies> {
        let rez = text(if let Some(fname) = &self.source_name {
            format!("Loaded {fname}")
        } else {
            "Unknown".to_string()
        });
        rez.width(Length::Fill).height(Length::Fill).into()
    }
}

impl DataSource for PCMFileSource {
    fn data(&self) -> Option<Vec<i16>> {
        Some(self.data.clone())
    }

    fn set_cache(&mut self, _data: Vec<i16>) {}
    fn clear_cache(&mut self) {}
    fn is_cached(&self) -> bool {
        true
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

pub trait DataSource {
    fn data(&self) -> Option<Vec<i16>>;
    fn set_cache(&mut self, data: Vec<i16>);
    fn clear_cache(&mut self);
    fn is_cached(&self) -> bool;
    fn selected_data(&self, selection: (usize, usize)) -> Option<Vec<i16>> {
        let data = self.data()?;
        let (from, to) = selection;
        Some(data[from.min(data.len())..to.min(data.len())].to_vec())
    }
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn play_audio(&self) {
        play_i16_audio(
            &self.data().expect("played only from cached"),
            self.sample_rate(),
            self.channels(),
        )
    }
    fn save_wav(&self) {
        save_wav(
            &self.data().expect("saved only from cached"),
            self.sample_rate(),
            self.channels(),
        )
    }
}

pub enum DataSourceType {
    RawData(PCMFileSource),
    WavePage(WaveformPage),
}

impl Default for DataSourceType {
    fn default() -> Self {
        Self::RawData(PCMFileSource::default())
    }
}

impl DataSource for DataSourceType {
    fn data(&self) -> Option<Vec<i16>> {
        match self {
            DataSourceType::RawData(x) => x.data(),
            DataSourceType::WavePage(x) => x.data(),
        }
    }

    fn set_cache(&mut self, data: Vec<i16>) {
        match self {
            DataSourceType::RawData(x) => x.set_cache(data),
            DataSourceType::WavePage(x) => x.set_cache(data),
        }
    }

    fn clear_cache(&mut self) {
        match self {
            DataSourceType::RawData(x) => x.clear_cache(),
            DataSourceType::WavePage(x) => x.clear_cache(),
        }
    }

    fn is_cached(&self) -> bool {
        match self {
            DataSourceType::RawData(x) => x.is_cached(),
            DataSourceType::WavePage(x) => x.is_cached(),
        }
    }

    fn sample_rate(&self) -> u32 {
        match self {
            DataSourceType::RawData(x) => x.sample_rate(),
            DataSourceType::WavePage(x) => x.sample_rate(),
        }
    }

    fn channels(&self) -> u16 {
        match self {
            DataSourceType::RawData(x) => x.channels(),
            DataSourceType::WavePage(x) => x.channels(),
        }
    }
}

impl DataSourceType {
    pub fn new_wedge(len: usize, focus: i16) -> Self {
        Self::RawData(PCMFileSource::new_wedge(len, focus))
    }
    pub fn new_noisy(len: usize) -> Self {
        Self::RawData(PCMFileSource::new_noisy(len))
    }
    pub fn new_widh_data(
        data: Vec<i16>,
        sample_rate: u32,
        channels: u16,
        name: impl ToString,
    ) -> Self {
        Self::RawData(PCMFileSource::new_widh_data(
            data,
            sample_rate,
            channels,
            name,
        ))
    }
    pub fn save_wav(&self) {
        if let Self::WavePage(wave) = self {
            wave.save_wav()
        } else {
            panic!()
        }
    }
    pub fn new_wav_page(sr_n_ch: Option<(u32, u16)>) -> Self {
        let rez = WaveformPage::new(sr_n_ch);
        Self::WavePage(rez)
    }
    pub fn process_page_signal(&mut self, signal: WavePageSig, buffer: &mut Vec<i16>) {
        if let Self::WavePage(wave) = self {
            wave.process_page_signal(signal, buffer)
        } else {
            panic!()
        }
    }
    pub fn process_wave_drawer_sig(&mut self, signal: WaveDrawerSig) {
        if let Self::WavePage(wave) = self {
            wave.process_wave_drawer_sig(signal)
        } else {
            panic!()
        }
    }
    pub fn view(&self) -> Element<'_, MesDummies> {
        match self {
            Self::WavePage(wave) => wave.view(),
            Self::RawData(x) => x.view(),
        }
    }
}

#[test]
fn test() {
    let rng = (12, 15);
    let x: Vec<usize> = (0..10).collect();
    let y = x[rng.0.min(x.len())..rng.1.min(x.len())].to_vec();
    println!("{:?}\n{:?}", x, y);
}
