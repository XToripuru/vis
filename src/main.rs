use std::fs::File;
use std::io::{BufReader, BufRead};
use crossterm::event::KeyEvent;
use rodio::{Source, OutputStreamHandle, source::SamplesConverter, Decoder};
use rustfft::{FftPlanner, num_complex::Complex};
use apodize;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{stdout, Read, Write};
use yt_api::{
	search::{Error, ItemType, SearchList},
	ApiKey,
};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, event::{poll, read, Event, KeyCode},
    cursor::{self, *}, style::{self, *}, Result, execute, queue
};
use std::time::{Instant, Duration};

enum Message {
    Action(Option<String>),
    Song(Song),
    Queue(Song),
    Skip
}

const R_ANIM: &[char] = &['⠟','⠯','⠷','⠾','⠽','⠻'];

fn main() {

    let mut stdout = stdout();
    let (w, h) = terminal::size().unwrap();
    let (sx, rx) = channel::<Message>();

    stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
    stdout.execute(terminal::SetSize(w, h)).unwrap();
    stdout.execute(cursor::DisableBlinking).unwrap();
    stdout.execute(cursor::Hide).unwrap();

    //let song = fft("C:\\Users\\zerotwo\\Desktop\\ROCKSTAR.mp3");
    //song.play();

    let (w, h) = (w as usize, h as usize);
    let mut buff = vec![' '; w*h];

    let mut song: Option<Song> = None;
    let mut queue: Vec<Song> = vec![];

    let mut inp = String::new();
    let mut action: Option<String> = None;
    let mut start = Instant::now();
    let mut a_values = vec![0f32; w];
    let mut values = vec![0f32; w];
    let mut a_min = 0.0;
    let mut min = 0.0;
    let mut a_bar = 48.0;

    loop {

        let millis = start.elapsed().as_millis();

        if let Some(act) = &action {
            //text(w, h, &mut buff, act, 0, h-2);
            text(w, h, &mut buff, format!("{}", R_ANIM[(millis/200) as usize % R_ANIM.len()]).as_str(), 0, h-2);
        }

        //text(w, h, &mut buff, "\u{25DC}", 0, h-2);

        let mut flen = 1;
        if let Some(song) = &mut song {
            //if inp.is_empty() {
            text(w, h, &mut buff, &song.title, w/2 - song.title.len()/2 + 1, h-1);
            //}

            // center: w/2 - 1 & w/2

            a_bar += ((millis as f32 * 48.0 / song.len as f32) - a_bar) * 0.01;
            text(w, h, &mut buff, "[", w/2 - 1 - 24, h-3);
            text(w, h, &mut buff, "]", w/2 + 24, h-3);
            text(w, h, &mut buff, &"\\".repeat(a_bar as usize), w/2 - 1 - 24 + 1, h-3);
            text(w, h, &mut buff, format!("{:02}:{:02}", (millis/1000)/60, (millis/1000)%60).as_str(), w/2 - 25, h-2);
            text(w, h, &mut buff, format!("{:02}:{:02}", (song.len/1000)/60, (song.len/1000)%60).as_str(), w/2 + 20, h-2);

            while song.fft.len() > 0 && song.fft[song.fft.len() - 1].0 < start.elapsed().as_millis() as f32 * 0.001 {
                let (time, frag) = song.fft.pop().unwrap();
                for k in 0..w {
                    let fk = ((k+1) as f32 / 4.0) - (k/4) as f32;
                    values[k] = 
                    (
                        //frag[2+k/3 - 2] +
                        0.1 * frag[1+k/4 - 1] +
                        (1.0 - fk) * frag[1+k/4] +
                        fk * frag[1+k/4 + 1] +
                        0.1 * frag[1+k/4 + 2]
                    ).sqrt() * (1.0 + k as f32 / 120.0) * (h as f32 / 10.0);
                    if values[k] >= (h - 5) as f32 {
                        values[k] = (h - 5) as f32;
                    }
                }
            }
            flen = song.fft.len();
        }

        if flen == 0 {
            song = None;
        }

        if queue.len() != 0 {
            if song.is_none() {
                if let Some(song) = &song {
                    song.stop();
                }
                let s = queue.remove(0);
                s.play();
                start = Instant::now();
                song = Some(s);
            }
            text(w, h, &mut buff, format!("{} more", queue.len()).as_str(), 0, h-3);
        }

        min = 1024.0;
        for k in 0..w {
            if a_values[k] < min {
                min = a_values[k];
            }
        }

        a_min += (min - a_min) * 0.001;

        for k in 0..w {
            // for q in 0..(a_values[k] - min) as usize {
            //     buff[(h-5-q) * w + k] = '|';
            // }
            buff[(h-5-(a_values[k] - a_min) as usize) * w + k] = '•';

            for k in 0..w {
                a_values[k] += (values[k] - a_values[k]) * 0.0015;
            }
        }

        if let Ok(m) = rx.try_recv() {
            match m {
                Message::Song(mut s) => {
                    s.fft.reverse();
                    s.play();
                    start = Instant::now();
                    song = Some(s);
                },
                Message::Queue(mut s) => {
                    s.fft.reverse();
                    queue.push(s);
                }
                Message::Action(opt) => {
                    action = opt;
                }
                Message::Skip => {
                    if let Some(song) = &song {
                        song.stop();
                    }
                    song = None;
                    values = vec![0f32; w];
                }
                _ => {}
            }
        }

        text(w, h, &mut buff, &inp, 0, h-1);
        if inp.len() > 0 && millis%1000 < 500 {
            text(w, h, &mut buff, "_", inp.len(), h-1);
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
                Event::Key(k) => {
                    match k.code {
                        KeyCode::Char(c) => {
                            inp.push(c);
                        }
                        KeyCode::Backspace if inp.len() > 0 => {
                            inp.pop();
                        }
                        KeyCode::Enter if inp.len() > 0 => {
                            let mut tokens: Vec<String> = inp.split(" ").map(|s| s.to_owned()).collect();
                            let first = tokens.remove(0);
                            inp.clear();
                            let n: String = tokens.join(" ");
                            match first.as_str() {
                                "ff" if tokens.len() == 1 => {
                                    let sx = sx.clone();
                                    let file = tokens[0].clone();
                                    std::thread::spawn(move || {
                                        for title in std::io::BufReader::new(File::open(file).unwrap()).lines() {
                                            sx.send(Message::Action(Some(String::from("1/3"))));
                                            let x = search(title.unwrap().as_str()).unwrap();
                                            let url = &x[0].1;
                                            sx.send(Message::Action(Some(String::from("2/3"))));
                                            let path = format!("audio-{}.wav", url);
                                            if !std::path::Path::new(&path).exists() {
                                                download(url);
                                            }
                                            sx.send(Message::Action(Some(String::from("3/3"))));
                                            let mut s = fft(&path);
                                            s.title = x[0].0.clone()
                                            .replace("&amp;", "&")
                                            .replace("&#39;", "'")
                                            .replace("&quot;", "\"");
                                            sx.send(Message::Queue(s));
                                            sx.send(Message::Action(None));
                                        }
                                    });
                                }
                                "s" if tokens.len() == 0 => {
                                    sx.send(Message::Skip);
                                }
                                "q" => {
                                    let sx = sx.clone();
                                    std::thread::spawn(move || {
                                        sx.send(Message::Action(Some(String::from("1/3"))));
                                        let x = search(&n).unwrap();
                                        let url = &x[0].1;
                                        sx.send(Message::Action(Some(String::from("2/3"))));
                                        let path = format!("audio-{}.wav", url);
                                        if !std::path::Path::new(&path).exists() {
                                            download(url);
                                        }
                                        sx.send(Message::Action(Some(String::from("3/3"))));
                                        let mut s = fft(&path);
                                        s.title = x[0].0.clone()
                                        .replace("&amp;", "&")
                                        .replace("&#39;", "'")
                                        .replace("&quot;", "\"");
                                        sx.send(Message::Queue(s));
                                        sx.send(Message::Action(None));
                                    });
                                }
                                "p" => {
                                    let sx = sx.clone();
                                    if let Some(song) = &song {
                                        song.stop();
                                    }
                                    song = None;
                                    values = vec![0f32; w];
                                    std::thread::spawn(move || {
                                        sx.send(Message::Action(Some(String::from("1/3"))));
                                        let x = search(&n).unwrap();
                                        let url = &x[0].1;
                                        sx.send(Message::Action(Some(String::from("2/3"))));
                                        let path = format!("audio-{}.wav", url);
                                        if !std::path::Path::new(&path).exists() {
                                            download(url);
                                        }
                                        sx.send(Message::Action(Some(String::from("3/3"))));
                                        let mut s = fft(&path);
                                        s.title = x[0].0.clone()
                                        .replace("&amp;", "&")
                                        .replace("&#39;", "'")
                                        .replace("&quot;", "\"");
                                        sx.send(Message::Queue(s));
                                        sx.send(Message::Action(None));
                                    });
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        stdout.flush().unwrap();
    }
}

fn text(w: usize, h: usize, buff: &mut Vec<char>, s: &str, x: usize, y: usize) {
    for (k, c) in s.chars().enumerate() {
        let k = y * w + x + k;
        buff[k] = c;
    }
}

const API_KEY: &str = "AIzaSyD27XzkAIxC7W3FQ5Y6E2okjBYr--m7qd4";
//const API_KEY: &str = "AIzaSyBO1jhRNMjGoGUDWypVoVDiNavITIgHq5k";
fn search(query: &str) -> Option<Vec<(String,String)>> {
    futures::executor::block_on(async {
        let key = ApiKey::new(API_KEY);

        let result = SearchList::new(key)
            .q(query)
            .max_results(3)
            .item_type(ItemType::Video)
            .await;

        if let Ok(result) = result {
            let mut v = vec![];
            for r in &result.items {
                v.push((r.snippet.title.clone().unwrap(), r.id.video_id.clone().unwrap()));
            }
            Some(v)
        } else {
            None
        }
    })
}

fn download(url: &str) {
    std::fs::remove_file("audio.m4a");

    let child = std::process::Command::new("cmd.exe")
    .stdout(std::process::Stdio::piped())
    .arg("/C").arg(
        format!("yt-dlp --newline -f 140 -r 100.0M -o audio.m4a https://www.youtube.com/watch?v={}", url).as_str()
    ).spawn().unwrap().wait();

    let child = std::process::Command::new("cmd.exe")
        .arg("/C")
        .arg(format!("ffmpeg -loglevel panic -y -i audio.m4a audio-{}.wav", url))
        .spawn().unwrap().wait();

    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::Purge)).unwrap();
    stdout.flush().unwrap();
}

fn fft(path: &str) -> Song {

    let (sx, rx) = channel();

    let file = File::open(path).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let nb: rodio::source::SamplesConverter<rodio::Decoder<std::io::BufReader<std::fs::File>>, f32> 
    = source.convert_samples();

    let file = File::open(path).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let buf: rodio::source::Buffered<rodio::source::SamplesConverter<rodio::Decoder<std::io::BufReader<std::fs::File>>, f32>> 
    = source.convert_samples().buffered();

    let chan = nb.channels();
    let rate = nb.sample_rate();
    //println!("channels = {}", chan);
    //println!("sample rate (single-channel) = {}", rate);
    let sps = (rate as f32 / 60.0) as usize;
    //println!("sample rate (60x less) (single-channel) = {}", sps);
    let sr = (sps as u16 * chan) as usize;
    //println!("sample rate (60x less) (multi-channel) = {}", sr);

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(sps);

    let mut slices = vec![];

    let swindow = apodize::nuttall_iter(sps).collect::<Vec<f64>>();

    let mut buffer = vec![];
    for (k, b) in buf.enumerate() {
        if k != 0 && k%sr == 0 {
            fft.process(&mut buffer);
            slices.push(buffer);
            buffer = vec![];
        }
        if k%2 == 0 {
            buffer.push(Complex {re: b * swindow[(k%sr)/2 as usize] as f32, im: 0.0});
        }
    }

    let mut out: Vec<(f32, Vec<f32>)> = vec![];

    for (dt, slice) in slices.iter().enumerate() {
        let time = dt as f32 / 60.0;
        let mut frag = vec![];
        for (k, v) in slice.iter().enumerate() {
            let magnitude = (v.re*v.re + v.im*v.im).sqrt();
            frag.push(magnitude);
        }
        out.push((time, frag));
    }

    let path = path.to_owned();
    std::thread::spawn(move || {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        loop {
            if let Ok(m) = rx.recv() {
                match m {
                    1 => {
                        let file = File::open(&path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        let nb: rodio::source::SamplesConverter<rodio::Decoder<std::io::BufReader<std::fs::File>>, f32> 
                        = source.convert_samples();
                        stream_handle.play_raw(nb).unwrap();
                    }
                    2 => {
                        break;
                    }
                    _ => unreachable!()
                }
            }
        };
    });

    let len = (out[out.len() - 1].0 * 1000.0).round() as u64;


    Song {
        title: String::new(),
        fft: out,
        sx,
        len
    }
}

fn input() -> String {
    let mut r = String::new();
    match std::io::stdin().read_line(&mut r) {
        Ok(_) => {
            r.pop();
            r.pop();
            r
        },
        _ => panic!("Cannot get console input"),
    }
}

struct Song {
    title: String,
    fft: Vec<(f32, Vec<f32>)>,
    sx: Sender<u8>,
    len: u64
}

impl Song {
    fn play(&self) {
        self.sx.send(1);
    }
    fn stop(&self) {
        self.sx.send(2);
    }
}