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

    let files = FileDialog::new().set_directory(dir).pick_file();
    let file = match files {
        Some(s) => s,
        None => {
            println!("no file selected");
            return (Vec::new(), 0, 0);
        }
    };

    // file.parent()
    let dir = file
        .parent()
        .map(|f| f.to_str().unwrap_or_default())
        .unwrap_or_default();
    fs::write("./dirsave.txt", dir).expect("Unable to write file");

    // symphonia_decode(file);
    let rez = play_ogg::temst(file);

    println!("donezo");
    rez
}

#[allow(unused)]
mod old_shit {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    use std::{fs::File, path::PathBuf};

    #[allow(unused)]
    fn symphonia_decode(file: PathBuf) {
        // Create a media source. Note that the MediaSource trait is automatically implemented for File,
        // among other types.
        let file = Box::new(File::open(file).unwrap());

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(file, Default::default());

        // Create a hint to help the format registry guess what format reader is appropriate. In this
        // example we'll leave it empty.
        let hint = Hint::new();

        // Use the default options when reading and decoding.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // Probe the media source stream for a format.
        let probed = match symphonia::default::get_probe().format(
            &hint,
            mss,
            &format_opts,
            &metadata_opts,
        ) {
            Ok(o) => o,
            Err(Error::Unsupported(e)) => {
                println!("Unsuppoert format {e}");
                return;
            }
            e => {
                e.unwrap();
                return;
            }
        };

        // Get the format reader yielded by the probe operation.
        let mut format = probed.format;

        // Get the default track.
        let track = format.default_track().unwrap();

        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        let mut sample_count = 0;
        let mut sample_buf = None;

        loop {
            // Get the next packet from the format reader.
            let packet = match format.next_packet() {
                Ok(n) => n,
                // Err(Error::DecodeError(_)) => {
                //     println!("decode error idk");
                //     break;
                // }
                Err(e) => {
                    println!("decode error idk {:?}", e);
                    break;
                }
            };

            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // The decoded audio samples may now be accessed via the audio buffer if per-channel
                    // slices of samples in their native decoded format is desired. Use-cases where
                    // the samples need to be accessed in an interleaved order or converted into
                    // another sample format, or a byte buffer is required, are covered by copying the
                    // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                    // example below, we will copy the audio buffer into a sample buffer in an
                    // interleaved order while also converting to a f32 sample format.

                    // If this is the *first* decoded packet, create a sample buffer matching the
                    // decoded audio buffer format.
                    if sample_buf.is_none() {
                        // Get the audio buffer specification.
                        let spec = *audio_buf.spec();

                        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                        let duration = audio_buf.capacity() as u64;

                        // Create the f32 sample buffer.
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);

                        // The samples may now be access via the `samples()` function.
                        sample_count += buf.samples().len();
                        print!("\rDecoded {} samples", sample_count);
                    }
                }
                Err(Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }
    }

    #[allow(unused)]
    fn get_other_formats(file: PathBuf) {
        let data_maybe = audrey::open(file);
        let mut data = match data_maybe {
            Ok(d) => d,
            Err(e) => {
                println!("error is {:?}", e);
                return;
            }
        };
        let thing = data.frames::<[i16; 2]>();
        for i in thing.take(1000) {
            println!("{:?}", i);
        }
    }

    #[allow(unused)]
    fn get_mp3(file: PathBuf) {
        use minimp3::{Decoder, Error, Frame};
        let mut decoder = Decoder::new(File::open(file).unwrap());

        let mut idx = 0usize;
        loop {
            match decoder.next_frame() {
                Ok(Frame {
                    data,
                    channels,
                    sample_rate,
                    layer,
                    bitrate,
                }) => {
                    println!(
                        "{:4} Decoded {} samples, (s: {} , l: {} , b: {})",
                        idx,
                        data.len() / channels,
                        sample_rate,
                        layer,
                        bitrate
                    )
                }
                Err(Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
            idx += 1;
        }
    }
}
#[allow(unused)]
mod asynch_load {
    fn async_file() {
        // Spawn dialog on main thread
        let task = rfd::AsyncFileDialog::new().pick_file();
        // Await somewhere else
        execute(async {
            let file = task.await;
            if let Some(file) = file {
                // If you are on native platform you can just get the path
                #[cfg(not(target_arch = "wasm32"))]
                println!("{:?}", file.path());
                // If you care about wasm support you just read() the file
                file.read().await;
            }
        });
    }

    use std::future::Future;

    #[cfg(not(target_arch = "wasm32"))]
    fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
        // this is stupid... use any executor of your choice instead

        use iced::futures;
        std::thread::spawn(move || futures::executor::block_on(f));
    }
    #[cfg(target_arch = "wasm32")]
    fn execute<F: Future<Output = ()> + 'static>(f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }
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
        let decoder = Decoder::new(buffer).expect("decoded");
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
