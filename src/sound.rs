use std::time::{Instant, Duration};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, HashSet};
use std::sync::{
    Arc, Mutex, RwLock,
    mpsc::{Sender, Receiver},
    atomic::{AtomicBool, AtomicUsize, Ordering}
};

use crate::message::*;
use notify::{Watcher, RecursiveMode, DebouncedEvent};
use rodio::*;
use rand::prelude::*;
use rand::distributions::weighted::WeightedIndex;
use lazy_static::lazy_static;
use regex::Regex;

mod sound_manager; use sound_manager::SoundManager;
mod sound_channel; use sound_channel::SoundChannel;

lazy_static! {
    static ref FAULTY_ESCAPE: Regex = Regex::new(
        r"\\([^\.\+\*\?\(\)\|\[\]\{\}\^\$])"
    ).unwrap();

    static ref EMPTY_EXPR: Regex = Regex::new(
        r"(\|\(\)\))"
    ).unwrap();
}

#[derive(Clone)]
pub enum SoundFileType {
    IsPath(PathBuf),
    IsPlaylist(Vec<PathBuf>)
}

#[derive(Clone)]
pub struct SoundFile {
    pub r#type: SoundFileType,	// path to audio file with sound. OR list of paths
    pub weight: f32,	// controls likelihood of sound to be chosen. Default is 100.
    pub volume: f32,	// adjusts volume of sample. Can range from -40 to +6 decibles, default 0.
    pub random_balance: bool,	// if set to true will randomply distribute sound between stereo channels.
    pub delay: usize,	// number, delay before sound is played. In miliseconds, default 0.
    pub balance: f32,	// adjusts stereo channel, can range for -1 (full left) to 1 (full right).
}

#[derive(Clone)]
pub struct VolumeLock(Arc<RwLock<f32>>);
impl VolumeLock {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(1.0)))
    }
    pub fn get(&self) -> f32 {
        *self.0.read().unwrap()
    }
    pub fn set(&self, volume: f32) {
        *self.0.write().unwrap() = volume;
    }
}

pub struct SoundEntry {
    pub pattern: regex::Regex,	// regular expression matching log line
    pub channel: Option<Box<str>>,	// channel on which sound is played. sounds played on channel can be looped/stopped prematurely
    pub loop_attr: Option<bool>,	// "start" - sound start loop on channel until different sound is played on channel (if it is non-looped sound, loop will resume when it is done playing) or sound with "stop" is triggered.
    pub concurency: Option<usize>,	// number of councured sounds allowd to be played besides this sound. If currenty playing more than that, sound is ignored. In miliseconds, default unlimited.
    pub timeout: Option<usize>,	// number, timeout during which is sound going to be prevented from playing again. In miliseconds default 0.
    pub probability: Option<usize>,	 // percentage, Propablity that sound will be played. Default is always played.
    pub delay: Option<usize>,	// number, delay before sound is played. In miliseconds, default 0.
    pub halt_on_match: bool,	// boolean, if set to true, sound sense will stop processing long line after it was matched to this sound. Default false
    pub random_balance: bool,	// boolean, if set to true will randomply distribute sound betweem stereo channels.
    pub playback_threshold: u8,
    pub files: Vec<SoundFile>,
    pub weights: Vec<f32>,
    pub current_timeout: usize,
    pub recent_call: usize,
}

pub fn run(sound_rx: Receiver<SoundMessage>, ui_tx: Sender<UIMessage>) {
    let mut manager : Option<SoundManager> = None;
    let mut file : Option<File> = None;
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(notify_tx, Duration::from_millis(100)).unwrap();

    let mut prev = Instant::now();
    loop {
        for message in sound_rx.try_iter() {
            use SoundMessage::*;
            match message {
                ChangeGamelog(path) => {
                    watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
                    let mut file0 = File::open(&path).unwrap();
                    file0.seek(SeekFrom::End(0)).unwrap();
                    file = Some(file0);
                    ui_tx.send(UIMessage::LoadedGamelog).unwrap();
                }

                ChangeSoundpack(path) => {
                    manager = Some(SoundManager::new(&path, ui_tx.clone()));
                }

                message => if let Some(manager) = manager.as_mut() {
                    match message {
                        ChangeIgnoreList(path) => {
                            let file = &mut File::open(&path).unwrap();
                            let buf = &mut Vec::new();
                            file.read_to_end(buf).unwrap();
                            let list: Vec<Regex> = String::from_utf8_lossy(&buf).lines().filter_map(|expr| {
                                let processed = FAULTY_ESCAPE.replace_all(expr, "$1");
                                let processed = EMPTY_EXPR.replace_all(&processed, ")?");
                                Regex::new(&processed).ok()
                            }).collect();
                            manager.set_ignore_list(list);
                        }

                        VolumeChange(channel,volume) => {
                            manager.set_volume(&channel, volume * 0.01);
                        }

                        // SetCurrentPathsAsDefault => {
                        //     println!("SetCurrentPathAsDefault");
                        // }

                        SetCurrentVolumesAsDefault(file) => {
                            manager.set_current_volumes_as_default(file);
                        }
                        _ => (),
                    }
                }
            }
        }

        for event in notify_rx.try_iter() {
            if file.is_some() && manager.is_some() {
                let manager = manager.as_mut().unwrap();
                if let DebouncedEvent::Write(_path) = event {
                    let file = file.as_mut().unwrap();
                    let mut buf = Vec::new();
                    file.read_to_end(&mut buf).unwrap();
                    let lossy = String::from_utf8_lossy(&buf);
                    lossy.lines().for_each(|log| {
                        manager.process_log(log);
                    });
                }
            }
        }
        if let Some(manager) = manager.as_mut() {
            let current = Instant::now();
            let dt = current.duration_since(prev).as_millis() as usize;
            manager.maintain(dt);
            prev = current;
        }
        
        std::thread::sleep(Duration::from_millis(50));
    }
}