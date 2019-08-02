extern crate tinyfiledialogs as tfd;
extern crate notify;
extern crate regex;
extern crate rodio;
extern crate xml;
extern crate winit;
extern crate conrod;
extern crate rand;

use std::env;
use std::path;
use std::time::Duration;

use notify::{Watcher, RecommendedWatcher, RecursiveMode};

mod sound;
mod ui;

fn main() {
    // soundsense-rs.exe "(gamelog.exe file)" "(soundpacks directory)"
    let mut args = env::args();
    let first_arg = args.nth(1);
    let second_arg = args.nth(0);

    let gamelog_path = if let Some(path_str) = first_arg {
        Some(path::PathBuf::from(&path_str))
    } else {
        tfd::open_file_dialog(
            "Select gamelog.txt file.",
            "gamelog.txt",
            Some( (&["*.txt"], "*.txt") )
        ).map(|path_str| {
            path::PathBuf::from(&path_str)
        })
    };

    let sounds_path = if let Some(path_str) = second_arg {
        path::PathBuf::from(path_str)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut p = path::PathBuf::from(manifest_dir);
        p.push("soundpacks");
        p
    } else {
        path::PathBuf::from("soundpacks")
    };

    if !sounds_path.is_dir()  {
        println!("Invalid soundpacks directory!");
        return
    }
    if gamelog_path.is_none() {
        println!("No gamelog.txt path was provided!");
        return
    }

    let gamelog_path = gamelog_path.unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(&gamelog_path, RecursiveMode::Recursive).unwrap();
    let (a, b) = std::sync::mpsc::channel(); // main a->b sound
    let (c, d) = std::sync::mpsc::channel(); // main d<-c sound

    // Create sound loop thread.
    std::thread::Builder::new()
        .name("sound_thread".to_string())
        .spawn(move ||
            sound::sound_thread(
                &gamelog_path,
                &sounds_path,
                rx,
                b,
                c
            )
        ).unwrap();
    
    // Main Loop
    loop {
        std::thread::sleep(
            std::time::Duration::from_millis(100)
        )
    }
}