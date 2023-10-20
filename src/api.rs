use std::path::{Path, PathBuf};
use std::fs::{
    File
};
use std::error::Error;
use std::io::{BufReader, BufRead, Cursor};
use std::process::ChildStdout;
use std::time::Duration;

use rodio::{Source, OutputStreamHandle, source::SamplesConverter, Decoder};
use rustfft::{FftPlanner, num_complex::Complex};
use apodize;

pub fn search(query: impl AsRef<str>) -> Result<(), Box<dyn Error>> {
    let _ = std::fs::remove_file("audio.wav");
    let _ = std::fs::remove_file("audio.mp3");

    let _ = std::process::Command::new("cmd.exe")
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .arg("/C").arg(
        format!("yt-dlp -f ba --audio-format wav -o audio.wav ytsearch1:{}", query.as_ref()).as_str()
    ).spawn()?.wait()?;

    let _ = std::process::Command::new("cmd.exe")
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .arg("/C").arg("ffmpeg -i audio.wav -acodec libmp3lame audio.mp3").spawn()?.wait()?;

    Ok(())
}

pub fn fft(path: impl AsRef<Path>) -> Result<Spectrum, Box<dyn Error>> {

    let src = File::open(path)?;

    let source = Decoder::new_mp3(BufReader::new(src))?;

    let samples: SamplesConverter<Decoder<BufReader<File>>, f32> = source.convert_samples();


    let ch = samples.channels() as usize;
    let rate = samples.sample_rate();
    assert!(rate % 60 == 0); // assume sample rate is divisible by 60 so that we can stream each frame 60 times per second
    let size = (rate / 60) as usize;
    let msize = size as usize * ch;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(size);

    let mut slices = vec![];

    let nuttall = apodize::nuttall_iter(size as usize).map(|n| n as f32).collect::<Vec<f32>>();

    let samples = samples.buffered();

    // TODO buffer not needed bc.: process(&mut slices[a..b])
    let mut buffer = vec![];
    for (k, b) in samples.enumerate() {
        if k != 0 && k % msize == 0 {
            fft.process(&mut buffer);
            if buffer.len() != size {
                break
            }
            slices.append(&mut buffer);
        }
        if k % ch == 0 {
            buffer.push(Complex {re: b * nuttall[(k % msize)/2] as f32, im: 0.0});
        }
    }

    let out = slices
    .into_iter()
    .map(|v| (v.re * v.re + v.im * v.im).sqrt())
    .collect::<Vec<f32>>();

    // in buffer, frames are every `size`, 60 frames = 1sec

    Ok(Spectrum {
        inner: out,
        size: size as usize,
        fps: 60
    })
}

pub struct Spectrum {
    pub inner: Vec<f32>,
    pub size: usize,
    pub fps: usize
}