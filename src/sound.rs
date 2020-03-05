use std::time::{Instant, Duration};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering}
};
use std::error::Error;

use crate::message::*;
use crossbeam::{
    sync::ShardedLock,
    channel::{Sender, Receiver}
};
use rodio::*;
use rand::prelude::*;
use rand::distributions::weighted::WeightedIndex;
use lazy_static::lazy_static;
use regex::Regex;

mod sound_manager; use sound_manager::SoundManager;
mod sound_channel; use sound_channel::SoundChannel;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

lazy_static! {
    static ref FAULTY_ESCAPE: Regex = Regex::new(
        r"\\([^\.\+\*\?\(\)\|\[\]\{\}\^\$])"
    ).unwrap();

    static ref EMPTY_EXPR: Regex = Regex::new(
        r"(\|\(\)\))"
    ).unwrap();
}

/// Show if the SoundFile is a single sound, or a playlist of multiple sounds.
#[derive(Clone)]
pub enum SoundFileType {
    /// Contains a single file path.
    IsPath(PathBuf),
    /// Contains multiple file paths.
    IsPlaylist(Vec<PathBuf>)
}

/// A struct containing all the information about a SoundFile.
#[derive(Clone)]
pub struct SoundFile {
    /// Path to audio file with sound. OR list of paths
    pub r#type: SoundFileType,
    /// Controls likelihood of sound to be chosen. Default is 100.
    pub weight: f32,
    /// Adjusts volume of sample. Can range from -40 to +6 decibles, default 0.
    pub volume: f32,
    /// If set to true will randomply distribute sound between stereo channels.
    pub random_balance: bool,
    /// number, delay before sound is played. In miliseconds, default 0.
    pub delay: usize,
    /// Adjusts stereo channel, can range for -1 (full left) to 1 (full right).
    pub balance: f32,
}

/// A thread-safe wrapper around a volume(f32) volume.
/// Intended to be used by LoopPlayers and OneshotPlayers.
#[derive(Clone)]
pub struct VolumeLock(Arc<ShardedLock<f32>>);
impl VolumeLock {
    #[inline]
    pub fn new() -> Self {
        Self(Arc::new(ShardedLock::new(1.0)))
    }
    #[inline]
    pub fn get(&self) -> f32 {
        *self.0.read().unwrap()
    }
    #[inline]
    pub fn set(&self, volume: f32) {
        *self.0.write().unwrap() = volume;
    }
}

#[derive(Clone)]
pub struct IsPausedLock(Arc<AtomicBool>);
impl IsPausedLock {
    #[inline]
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    #[inline]
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn flip(&self) -> bool {
        self.0.fetch_nand(true, Ordering::SeqCst)
    }
}

/// A struct containing all the information about a Sound, such as regex patterns, channel, loopability, etc.
pub struct SoundEntry {
    /// regular expression matching log line
    pub pattern: regex::Regex,
    /// channel on which sound is played. sounds played on channel can be looped/stopped prematurely
    pub channel: Option<Box<str>>,
    /// "start" - sound start loop on channel until different sound is played on channel
    /// (if it is non-looped sound, loop will resume when it is done playing) or sound with "stop" is triggered.
    pub loop_attr: Option<bool>,
    /// number of councured sounds allowed to be played besides this sound.
    /// If currenty playing more than that, sound is ignored. In miliseconds, default unlimited.
    pub concurency: Option<usize>,
    /// number, timeout during which is sound going to be prevented from playing again. In miliseconds default 0.
    pub timeout: Option<usize>,
    /// percentage, Propablity that sound will be played. Default is always played.
    pub probability: Option<usize>,
    /// number, delay before sound is played. In miliseconds, default 0.
    pub delay: Option<usize>,
    /// boolean, if set to true, sound sense will stop processing long line after it was matched to this sound.
    /// Default false
    pub halt_on_match: bool,
    /// boolean, if set to true will randomply distribute sound betweem stereo channels.
    pub random_balance: bool,
    /// number, threashold used when filtering sound depending on level (currently not used)
    pub playback_threshold: u8,
    /// Collection of SoundFiles
    pub files: Vec<SoundFile>,
    /// Collection of each SoundFile's weight.
    pub weights: Vec<f32>,
    /// Timeout. While timeout, can't be played.
    pub current_timeout: usize,
    /// Number of times this SoundEntry has been called.
    pub recent_call: usize,
}

#[non_exhaustive]
#[derive(Copy, Clone, PartialEq)]
pub enum ChannelPlayType {
    All,
    SingleEager,
    SingleLazy,
}

pub struct ChannelSetting {
    play_type: ChannelPlayType,
}

/// The sound thread function.
pub fn run(sound_rx: Receiver<SoundMessage>, ui_tx: Sender<UIMessage>) {
    // Outer loop. Restarts the inner loop if an error occured, but didn't panic.
    loop {
        info!("(Re)Starting sound thread.");
        // SoundManager
        let mut manager : Option<SoundManager> = None;
        // BufReader for the gamelog.
        let mut buf_reader : Option<BufReader<File>> = None;
        // Current time for delta time calculation.
        let mut prev = Instant::now();

        // Arguably the most front-heavy if statement I ever wrote.
        if let Err(error) = || -> Result<()> {
            // Inner loop. Will return an Error if something wrong happens.
            loop {
                // Read SoundMessages sent from the UI.
                for message in sound_rx.try_iter() {
                    use SoundMessage::*;
                    match message {
                        ChangeGamelog(path) => {
                            let mut file0 = File::open(&path)?;
                            file0.seek(SeekFrom::End(0))?;
                            buf_reader = Some(BufReader::new(file0));
                            ui_tx.send(UIMessage::LoadedGamelog)?;
                        }

                        ChangeSoundpack(path) => {
                            if let Some(prev_manager) = manager.take() {
                                prev_manager.finish();
                            }
                            manager.replace(
                                SoundManager::new(&path, ui_tx.clone())?
                            );
                        }

                        // These types of messages require a manager.
                        message => if let Some(manager) = manager.as_mut() {
                            match message {
                                ChangeIgnoreList(path) => {
                                    let file = &mut File::open(&path)?;
                                    let buf = &mut Vec::new();
                                    file.read_to_end(buf)?;
                                    let list: Vec<Regex> = String::from_utf8_lossy(&buf).lines().filter_map(|expr| {
                                        let processed = FAULTY_ESCAPE.replace_all(expr, "$1");
                                        let processed = EMPTY_EXPR.replace_all(&processed, ")?");
                                        Regex::new(&processed).ok()
                                    }).collect();
                                    manager.set_ignore_list(list)?;
                                }

                                VolumeChange(channel,volume) => {
                                    manager.set_volume(&channel, volume * 0.01)?;
                                }

                                ThresholdChange(channel,threshold) => {
                                    trace!("Set channel {} threshold to {}", channel, threshold);
                                    manager.set_threshold(&channel, threshold)?;
                                }

                                SkipCurrentSound(channel) => {
                                    trace!("Skip Current Sound in {}", channel);
                                    manager.skip(&channel)?;
                                }

                                PlayPause(channel) => {
                                    trace!("Play/Pause {}", channel);
                                    manager.play_pause(&channel)?;
                                }

                                SetCurrentVolumesAsDefault(file) => {
                                    manager.set_current_volumes_as_default(file)?;
                                }
                                _ => (),
                            }
                        }
                    }
                }
                let current = Instant::now();
                if let Some(manager) = &mut manager {
                    if let Some(buf_reader) = &mut buf_reader {
                        let dt = current.duration_since(prev).as_millis() as usize;
                        for log in buf_reader
                            .lines()
                            .filter_map(|l| l.ok())
                        {
                            manager.process_log(&log)?;
                        }
                        manager.maintain(dt)?;
                    }
                }
                prev = current;
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }(){// LOOK, A BUTTERFLY!
            // If an error occurred and was caught, send the error message to the UI.
            // Return to the outer loop, which will then restart the inner loop.
            let mut error_message = "The soundthread restarted due to this error:\n".to_string();
            error_message.push_str(&error.to_string());
            ui_tx.send(
                UIMessage::SoundThreadPanicked(
                    "SoundThread Error".to_string(),
                    error_message,
                )
            ).unwrap();
            error!("SoundThreadError:\n{:?}", error);
        }
    }
}
