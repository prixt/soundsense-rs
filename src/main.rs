extern crate tinyfiledialogs as tfd;
extern crate notify;
extern crate regex;
extern crate rodio;

use std::env;
use std::fs;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::io::prelude::*;
use std::io::SeekFrom;

use notify::{Watcher, RecommendedWatcher, RecursiveMode, DebouncedEvent};

fn watch(path: &str) -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    watcher.watch(path, RecursiveMode::Recursive)?;

    let file = &mut fs::File::open(path)?;
    let mut cursor_pos = file.seek(SeekFrom::End(0))?;
    let buffer = &mut Vec::new();
    loop {
        match rx.recv() {
            Ok(event) => {
                if let DebouncedEvent::Write(_) = event {
                    file.seek(SeekFrom::Start(cursor_pos))?;
                    file.read_to_end(buffer)?;
                    cursor_pos = file.seek(SeekFrom::Current(0))?;
                    let messages = String::from_utf8_lossy(&buffer);
                    for line in messages.lines() {
                        println!(
                            ">> {}",
                            line.trim()
                        );
                    }
                    buffer.clear()
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn main() {
    let path_str = if let Some(path_str) = env::args().nth(1) {
        Some(path_str)
    } else {
        tfd::open_file_dialog(
            "Select gamelog.txt",
            "./gamelog.txt",
            Some(
                (&["*.txt"], "*.txt")
            )
        )
    };

    if let Some(path) = path_str {
        watch(&path).unwrap();
    }
}