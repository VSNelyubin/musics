use rfd::FileDialog;
use std::fs;

#[allow(unused)]
fn test_env() {
    for (key, value) in std::env::vars().filter(|(k, _)| k == "GTK_PATH") {
        println!("{key}: {value}");
        // if key == "GTK_PATH" {
        //     break;
        // }
    }
    println!("\n");
}

pub fn find_file() -> (Vec<i16>, u32, u16) {
    let dir = fs::read("./dirsave.txt")
        .map(|v| String::from_utf8(v).unwrap_or("./".to_string()))
        .unwrap_or("./".to_string());

    let files = FileDialog::new()
        .set_directory(dir)
        .add_filter("acceptable formats", &["wav", "mp3", "ogg"])
        .pick_file();
    let file = match files {
        Some(s) => s,
        None => {
            println!("no file selected");
            return (Vec::new(), 0, 0);
        }
    };

    let dir = file
        .parent()
        .map(|f| f.to_str().unwrap_or_default())
        .unwrap_or_default();
    fs::write("./dirsave.txt", dir).expect("Unable to write file");

    play_ogg::temst(file)
}

#[allow(unused_imports, unused)]
mod play_ogg {
    use std::any::Any;
    use std::io::{BufReader, Read};
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use audrey::dasp_sample::ToSample;
    use rodio::{Decoder, Source};
    use symphonia::core::conv::IntoSample;

    pub fn temst(path: PathBuf) -> (Vec<i16>, u32, u16) {
        // let (_stream, _stream_handle) = rodio::OutputStream::try_default().unwrap();

        let file = std::fs::File::open(path).unwrap();
        let mut buffer = BufReader::new(file);
        // let mut all_buf = Vec::new();
        // let x = buffer.read_to_end(&mut all_buf);
        // println!("len is {:?} bytes", x);
        let decoder = if let Ok(x) = Decoder::new(buffer) {
            x
        } else {
            println!("file couldt not be decoded");
            return (Vec::new(), 0, 0);
        };
        let (sample_rate, channels) = (decoder.sample_rate(), decoder.channels());
        let data: Vec<i16> = decoder.collect();
        // let beep1 = match _stream_handle.play_once(buffer) {
        //     Ok(o) => o,
        //     Err(e) => {
        //         println!("cannot open file: {e}");
        //         return;
        //     }
        // };
        // beep1.set_volume(1.0);
        // // println!("Started beep1");
        // // beep1.detach();
        // thread::sleep(Duration::from_millis(1500));
        // drop(beep1);
        // // println!("Stopped beep1");
        (data, sample_rate, channels)
    }
}
