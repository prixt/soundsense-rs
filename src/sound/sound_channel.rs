use super::*;
use source::Spatial;

mod loop_player;
mod oneshot_player;

use loop_player::LoopPlayer;
use oneshot_player::OneshotPlayer;

/// Struct responsible for containing currently playing sounds.
/// "music" and "weather" channels can play only one sound at a time.
pub struct SoundChannel {
    looping: LoopPlayer,
    one_shots: OneshotPlayer,
    local_volume: VolumeLock,
    delay: usize,
    local_is_paused: IsPausedLock,
    threshold: u8,
    pub play_type: ChannelPlayType,
}

impl SoundChannel {
    /// Create a new SoundChannel.
    #[inline]
    pub fn new(device: &Device, name: &str, total_volume: VolumeLock, total_is_paused: IsPausedLock) -> Self {
        let local_volume = VolumeLock::new();
        let local_is_paused = IsPausedLock::new();
        Self {
            looping : LoopPlayer::new(
                device,
                local_volume.clone(),
                total_volume.clone(),
                local_is_paused.clone(),
                total_is_paused.clone(),
            ),
            one_shots : OneshotPlayer::new(
                local_volume.clone(),
                total_volume,
                local_is_paused.clone(),
                total_is_paused,
            ),
            local_volume,
            delay : 0,
            play_type: {
                if name == "weather" || name == "music" {
                    ChannelPlayType::SingleEager
                }
                else {
                    ChannelPlayType::All
                }
            },
            local_is_paused,
            threshold: 4,
        }
    }

    /// Maintain this channel.
    /// Maintain looping player, tick down delay, cleanup oneshots.
	pub fn maintain(&mut self, rng: &mut ThreadRng, dt: usize) {
		let delay = self.delay.saturating_sub(dt);
        self.delay = delay;
        if self.delay > 0 {
            self.one_shots.pause()
        }
        self.one_shots.maintain();
        if self.one_shots.is_empty() && delay == 0
            && !self.looping.is_stopped() {
            self.looping.play();
            self.looping.set_volume(1.0);
        }
        else {
            self.looping.pause()
        }
        self.looping.maintain(rng);
	}

    /// Change the loop.
    /// If "music" or "weather", stop all oneshots.
    pub fn change_loop(&mut self, device: &Device, files: &[SoundFile], delay: usize, rng: &mut ThreadRng) {
        if self.play_type == ChannelPlayType::SingleLazy {
            if self.len() != 0 {
                return
            }
        }
        else if self.play_type == ChannelPlayType::SingleEager {
            self.one_shots.stop();
        }
        self.looping.change_loop(device, files, rng);
        self.delay = delay;
        self.maintain(rng, 0);
    }

    pub fn stop_loop(&mut self, delay: usize) {
        self.looping.stop();
        self.delay = delay;
    }

    pub fn skip(&mut self) {
        self.looping.skip();
        self.one_shots.stop();
    }

    pub fn play_pause(&mut self) -> bool {
        self.local_is_paused.flip()
    }

    pub fn finish(&mut self) {
        self.looping.stop();
        self.one_shots.stop();
    }

    /// Play a oneshot.
    /// Will make other oneshots 50% quieter.
    /// If "music" or "weather", pauses loop and stops other oneshots. 
    pub fn add_oneshot(&mut self, device: &Device, file: &SoundFile, delay: usize, rng: &mut ThreadRng) {
        if self.play_type == ChannelPlayType::SingleLazy {
            if self.len() != 0 {
                return
            }
        }
        else if self.play_type == ChannelPlayType::SingleEager {
            self.looping.pause();
            self.one_shots.stop();
        }

        self.one_shots.play();
        for idx in 0..self.one_shots.len() {
            let current_vol = self.one_shots.get_volume(idx);
            self.one_shots.set_volume(idx, current_vol * 0.5);
        }
        self.looping.set_volume(0.25);
        let mut data = get_soundfiles(file, rng);
        match data.len() {
            0 => (),
            1 => {
                let (source, volume, balance) = data.remove(0);
                self.one_shots.add_source(device, source, volume, balance);
            }
            _ => {
                let (source, volume, balance)
                    = data.remove(rng.gen_range(0, data.len()));
                self.one_shots.add_source(device, source, volume, balance);
            }
        }
        self.delay = delay;
    }

    #[inline]
    pub fn set_local_volume(&mut self, local_volume: f32) {
        self.local_volume.set(local_volume);
    }
    #[inline]
    pub fn get_local_volume(&self) -> f32 {
        self.local_volume.get()
    }

    #[inline]
    pub fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold;
    }
    #[inline]
    pub fn get_threshold(&mut self) -> u8 {
        self.threshold
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.one_shots.len() + self.looping.len()
    }
}

/// Get a Vector of (source, volume, balance) from a SoundFile.
/// Note that non-playlist files will just return a 1-length Vector.
fn get_soundfiles(soundfile: &SoundFile, rng: &mut ThreadRng)
    -> Vec<(rodio::decoder::Decoder<std::fs::File>, f32, f32)>
{
    let volume = soundfile.volume;
    let balance = if soundfile.random_balance {
        rng.gen_range(-1.0, 1.0)
    } else {
        soundfile.balance
    };
    match soundfile.r#type {
        SoundFileType::IsPath(ref path) => {
            if let Some(source) = get_source(path) {
                return vec![ (source, volume, balance) ]
            }
        }
        SoundFileType::IsPlaylist(ref paths) => {
            if let Some(source) = get_source(&paths.choose(rng).unwrap())
            {
                return vec![ (source, volume, balance) ]
            }
        }
    }
    vec![]
}

/// Check if the file at the give path is a valid sound source.
/// Otherwise, return a None. 
fn get_source(path: &Path) -> Option<rodio::decoder::Decoder<std::fs::File>> {
    let f = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Path {} is invalid: {}", path.display(), e);
            warn!("Will ignore this source.");
            return None
        }
    };
    let source = Decoder::new(f);
    match source {
        Ok(source) => {
            Some(source)
        },
        Err(e) => {
            warn!("Failed to assert {}: {}", path.display(), e);
            warn!("Will ignore this source.");
            None
        }
    }
}
