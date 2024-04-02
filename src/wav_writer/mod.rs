use std::fs::{self, File};

use rfd::FileDialog;
use wav::{Header, WAV_FORMAT_PCM};

pub fn save_wav(data: &[i16], sample_rate: u32, channels: u16) {
    let dir = fs::read("./dirsave.txt")
        .map(|v| String::from_utf8(v).unwrap_or("./".to_string()))
        .unwrap_or("./".to_string());

    let files = FileDialog::new().set_directory(dir).save_file();
    let file = match files {
        Some(s) => s,
        None => {
            println!("no file selected");
            return;
        }
    };

    let header = Header::new(WAV_FORMAT_PCM, channels, sample_rate, 16);
    let mut out_file = File::create(file).unwrap();
    wav::write(
        header,
        &wav::BitDepth::Sixteen(data.to_vec()),
        &mut out_file,
    )
    .unwrap();
}
