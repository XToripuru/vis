use std::error::Error;
use std::path::Path;
use std::thread::{sleep, JoinHandle};
use std::time::{Instant, Duration};
use std::io::{stdout, Read, Write};
use std::fs::{File};
use std::io::{BufReader};
use api::Spectrum;
use crossterm::event::KeyEventKind;
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, event::{poll, read, Event, KeyCode},
    cursor::{self, *}, style::{self, *}, execute, queue
};
use rodio::{OutputStream, Sink, Decoder};

mod api;

fn main() {

    let mut player = None;

    let mut stdout = stdout();
    let (w, h) = terminal::size().unwrap();

    stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
    stdout.execute(terminal::SetSize(w, h)).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();
    stdout.execute(cursor::Hide).unwrap();

    let (w, h) = (w as usize, h as usize);
    let mut buff = vec![' '; w * h];

    let mut input = String::new();
    let mut task = None::<JoinHandle<Spectrum>>;
    
    let mut smooth = vec![0.0; w];

    loop {

        if matches!(task, Some(ref task) if task.is_finished()) {
            let Some(task) = task.take() else { panic!() };

            let (stream, handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&handle).unwrap();
            let file = File::open("audio.mp3").unwrap();
            let start = Instant::now();
            sink.append(Decoder::new(BufReader::new(file)).unwrap());

            player = Some(Player {
                spectrum: task.join().unwrap(),
                start,
                stream,
                sink
            });
        }

        if let Some(ref mut player) = player {
            let elapsed = player.start.elapsed().as_millis() as usize;
            let frame = &player.spectrum.inner[player.spectrum.size * (elapsed as f32 / (1000.0 / player.spectrum.fps as f32)) as usize..][..player.spectrum.size];

            for q in 0..w {
                let mut value = 0.0;
                for k in (0..(frame.len() / w)).map(|m| q + m * w) {
                    let p = k as f32 * 0.25;
                    let r = 4.0;
                    let range =
                    (p - r).clamp(0.0, frame.len() as f32) as usize
                    ..
                    (p + r).clamp(0.0, frame.len() as f32) as usize;

                    for q in range {
                        let d = p - q as f32;
                        value += (h as f32 / 16.0) * (1.0 + q as f32 / 8.0) * frame[q] / (1.0 + d * d);
                    }

                }
                smooth[q] += (value.sqrt() - smooth[q]) * 0.2;
            }

            for x in 0..w {
                let y = h - 1 - (smooth[x]).clamp(0.0, (h - 2) as f32) as usize;
                buff[x + y * w] = '•';
            }
        }

        for (k, ch) in input.chars().chain(['_']).enumerate() {
            buff[k + (h - 1) * w] = ch;
        }

        for y in 0..h {
            for x in 0..w {
                let k = y * w + x;
                queue!(
                    stdout,
                    MoveTo(x as u16, y as u16),
                    Print(buff[k]),
                    SetAttribute(Attribute::Reset)
                ).unwrap();
                buff[k] = ' ';
            }
        }

        if let Ok(true) = poll(std::time::Duration::from_millis(0)) {
            match read().unwrap() {
                Event::Key(k) => if k.kind == KeyEventKind::Press {
                    match k.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace if input.len() > 0 => {
                            let _ = input.pop();
                        }
                        KeyCode::Enter if input.len() > 0 && task.is_none() => {
                            let query = input.replace(" ", "_");
                            input.clear();
                            task = Some(std::thread::spawn(move || {
                                //println!("Searching");
                                api::search(query).unwrap();
                            
                                //println!("FFT");
                                api::fft("audio.mp3").unwrap()
                            }));
                        }
                        _ => {}
                    }
                }
                Event::Paste(s) => {
                    input.push_str(&s);
                }
                _ => {}
            }
        }
    
    }

}

struct Player {
    spectrum: Spectrum,
    start: Instant,
    stream: OutputStream,
    sink: Sink
}