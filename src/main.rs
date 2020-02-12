// #![windows_subsystem = "windows"]
use std::env;
use std::sync::mpsc::channel;
use std::path::PathBuf;
use regex::Regex;

mod sound;
mod ui;
mod message;
//mod download;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optopt("l", "gamelog", "Path to the gamelog.txt file.", "LOG_FILE");
    opts.optopt("p", "soundpack", "Path to the soundpack directory.", "PACK_DIR");
    opts.optopt("i", "ignore", "Path to the ignore.txt file.", "IGNORE_FILE");

    let conf = dirs::config_dir()
        .and_then(|mut p| {
            p.push("soundsense-rs/conf.ini");
            std::fs::read_to_string(p).ok()
        });

    let matches = opts.parse(&args[1..]).unwrap();
    let gamelog_path = matches.opt_str("l").and_then(|path| {
        let path = PathBuf::from(path);
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    })
    .or_else(|| if let Some(conf_txt) = &conf {
        Regex::new("gamelog=(.+)").unwrap()
            .captures(&conf_txt)
            .and_then(|c| c.get(1))
            .map(|m| PathBuf::from(m.as_str()))
    } else {
        None
    })
    .or_else(|| {
        let path = PathBuf::from("./gamelog.txt");
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    });
    let soundpack_path = matches.opt_str("p").and_then(|path| {
        let path = PathBuf::from(path);
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    })
    .or_else(|| if let Some(conf_txt) = &conf {
        Regex::new("soundpack=(.+)").unwrap()
            .captures(&conf_txt)
            .and_then(|c| c.get(1))
            .map(|m| PathBuf::from(m.as_str()))
    } else {
        None
    })
    .or_else(|| {
        let path = PathBuf::from("./soundpack");
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    });
    let ignore_path = matches.opt_str("i").and_then(|path| {
        let path = PathBuf::from(path);
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    })
    .or_else(|| if let Some(conf_txt) = &conf {
        Regex::new("ignore=(.+)").unwrap()
            .captures(&conf_txt)
            .and_then(|c| c.get(1))
            .map(|m| PathBuf::from(m.as_str()))
    } else {
        None
    })
    .or_else(|| {
        let path = PathBuf::from("./ignore.txt");
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    });

    let (sound_tx, sound_rx) = channel();
    let (ui_tx, ui_rx) = channel();
    std::thread::Builder::new()
        .name("sound_thread".to_string())
        .spawn(move || sound::run(sound_rx, ui_tx)).unwrap();
    ui::run(sound_tx, ui_rx, gamelog_path, soundpack_path, ignore_path);
}