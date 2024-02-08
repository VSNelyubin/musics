use std::{
    sync::{Arc, Mutex},
    thread,
};

use rodio::{buffer::SamplesBuffer, OutputStream, Sink, Source};
pub fn play_i16_audio(data: &[i16], sample_rate: u32, channels: u16) {
    let loc_data = data.to_vec();
    let buffer = SamplesBuffer::new(channels, sample_rate, loc_data);
    let buffer = buffer.convert_samples::<f32>();
    let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
    // println!("playing");
    // _stream_handle.play_raw(buffer).unwrap();

    let sink = Sink::try_new(&_stream_handle).unwrap();

    let sinc = Arc::new(Mutex::new(sink));

    let sink = sinc.lock().unwrap();

    sink.append(buffer);

    let sink_clone = Arc::clone(&sinc);

    let thread = thread::spawn(move || {
        let sink = sink_clone.lock().unwrap();

        sink.sleep_until_end();

        // thread::sleep(Duration::from_secs(2));
    });

    thread.join().unwrap();
}
