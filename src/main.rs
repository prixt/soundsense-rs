#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use std::env;
use std::path::PathBuf;
use regex::Regex;
#[macro_use] extern crate log;
use crossbeam::channel::unbounded as channel;

mod sound;
mod ui;
mod message;

/// How SoundSense-RS works:
/// 1. Dwarf Fortress(&DFHack) writes into gamelog.txt
/// 2. In the Sound thread, every loop, the SoundManager reads the newly written lines.
/// 3. The SoundManager iterates through the SoundEntries, and checks if any of their patterns match.
/// 4. If a pattern matches, play the SoundEntry's SoundFiles on the appropriate SoundChannel.
/// 
/// All the while the UI thread handles user input and sends SoundMessage to the SoundThread
/// through a Sender<SoundMessage>, while the Sound thread sends UIMessages to the UI through
/// a Sender<UIMessage>.

fn main() {
    // Setup and initialize the env_logger.
    let env = env_logger::Env::default()
        .filter_or("SOUNDSENSE_RS_LOG", "warn")
        .write_style_or("SOUNDSENSE_RS_LOG_STYLE", "always");
    env_logger::Builder::from_env(env)
        .format_module_path(false)
        .format_timestamp_millis()
        .init();
    info!("Starting SoundSense-RS");
    
    // Setup getopts style argument handling.
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optopt("l", "gamelog", 
        "Path to the gamelog.txt file. (Default: .\\gamelog.txt)", "LOG_FILE")
        .optopt("p", "soundpack", 
        "Path to the soundpack directory. (Default: .\\soundpack)", "PACK_DIR")
        .optopt("i", "ignore", 
        "Path to the ignore.txt file. (Default: .\\ignore.txt)", "IGNORE_FILE")
        .optflag("", "no-config", 
        "Don't read config files on start. Will use the given paths, or soundsense-rs defaults.");

    // If there are errors in the arguments, print the usage of SoundSense-RS and quit.
    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(e) => {
            error!("{}", e);
            println!("{}", opts.usage("SoundSense-RS"));
            return
        }
    };

    // Check if there are config files available.
    // If so, read `soundsense-rs/default-paths.ini`.
    let config = if !matches.opt_present("no-config") {
        dirs::config_dir()
            .and_then(|mut p| {
                p.push("soundsense-rs/default-paths.ini");
                std::fs::read_to_string(p).ok()
            })
    } else {None};

    let gamelog_path = matches
        .opt_str("l")
        // If a path is given, and is a file, use that as the gamelog.
        .and_then(|path| {
            let path = PathBuf::from(path);
            if path.is_file() {Some(path)} else {None}
        })
        // Else if config file contains path to the gamelog, use that as the gamelog.
        .or_else(||
            config.as_ref()
                .and_then(|config_txt|
                        Regex::new("gamelog=(.+)").unwrap()
                            .captures(&config_txt)
                            .and_then(|c| c.get(1))
                            .map(|m| PathBuf::from(m.as_str()))
                            .filter(|p| p.is_file())
                )
        )
        // Else try to find `gamelog.txt` in the current working directory.
        // Otherwise, just return None.
        .or_else(|| {
            let mut path = env::current_dir()
                .expect("Error finding current working directory.");
            path.push("gamelog.txt");
            if path.is_file() {Some(path)} else {None}
        });
    let soundpack_path = matches
        .opt_str("p")
        // If a path is given, and is a directory, use that as the soundpack.
        .and_then(|path| {
            let path = PathBuf::from(path);
            if path.is_dir() {Some(path)} else {None}
        })
        // Else if config file contains path to the soundpack, use that as the soundpack.
        .or_else(||
            config.as_ref()
                .and_then(|config_txt|
                    Regex::new("soundpack=(.+)").unwrap()
                        .captures(&config_txt)
                        .and_then(|c| c.get(1))
                        .map(|m| PathBuf::from(m.as_str()))
                        .filter(|p| p.is_dir())
                )
        )
        // Else try to find `soundpack` directory in the current working directory.
        // Otherwise, just return None.
        .or_else(|| {
            let mut path = env::current_dir()
                .expect("Error finding current working directory.");
            path.push("soundpack");
            if path.is_dir() {Some(path)} else {None}
        });
    let ignore_path = matches
        .opt_str("i")
        // If a path is given, and is a file, use that as the ignore list.
        .and_then(|path| {
            let path = PathBuf::from(path);
            if path.is_file() {Some(path)} else {None}
        })
        // Else if config file contains path to the ignore list, use that as the ignore list.
        .or_else(||
            config.as_ref()
                .and_then(|config_txt|
                    Regex::new("ignore=(.+)").unwrap()
                        .captures(&config_txt)
                        .and_then(|c| c.get(1))
                        .map(|m| PathBuf::from(m.as_str()))
                        .filter(|p| p.is_file())
                )
        )
        // Else try to find `ignore.txt` in the current working directory.
        // Otherwise, just return None.
        .or_else(|| {
            let mut path = env::current_dir()
                .expect("Error finding current working directory.");
            path.push("ignore.txt");
            if path.is_file() {Some(path)} else {None}
        });

    let (sound_tx, sound_rx) = channel();
    let (ui_tx, ui_rx) = channel();
    
    // Build and spawn the Sound thread.
    std::thread::Builder::new()
        .name("sound_thread".to_string())
        .spawn(move || sound::run(sound_rx, ui_tx)).unwrap();
    // Start the UI thread.
    ui::run(sound_tx, ui_rx, gamelog_path, soundpack_path, ignore_path);
}
