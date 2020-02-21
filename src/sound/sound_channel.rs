use super::*;

#[allow(unused_imports)]
use source::Spatial;

mod loop_player;
mod oneshot_player;

use loop_player::LoopPlayer;
use oneshot_player::OneshotPlayer;

pub struct SoundChannel {
    looping: LoopPlayer,
    one_shots: OneshotPlayer,
    local_volume: VolumeLock,
    delay: usize,
    only_one_sound: bool,
}

impl SoundChannel {
    #[inline]
    pub fn new(device: &Device, name: &str, total_volume: VolumeLock) -> Self {
        let local_volume = VolumeLock::new();
        Self {
            looping : LoopPlayer::new(device, local_volume.clone(), total_volume.clone()),
            one_shots : OneshotPlayer::new(local_volume.clone(), total_volume),
            local_volume,
            delay : 0,
            only_one_sound: name == "weather" || name == "music",
        }
    }

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

    pub fn change_loop(&mut self, device: &Device, files: &[SoundFile], delay: usize, rng: &mut ThreadRng) {
        self.looping.change_loop(device, files, rng);
        self.delay = delay;
        self.maintain(rng, 0);
        if self.only_one_sound {
            self.one_shots.stop();
        }
    }

    pub fn stop_loop(&mut self, delay: usize) {
        self.looping.stop();
        self.delay = delay;
    }

    pub fn add_oneshot(&mut self, device: &Device, file: &SoundFile, delay: usize, rng: &mut ThreadRng) {
        if self.only_one_sound {
            self.looping.pause();
            self.one_shots.stop();
        }
        self.one_shots.play();
        for idx in 0..self.one_shots.len() {
            let current_vol = self.one_shots.get_volume(idx);
            self.one_shots.set_volume(idx, current_vol * 0.75);
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
    pub fn len(&self) -> usize {
        self.one_shots.len() + self.looping.len()
    }
}

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

fn get_source(path: &Path) -> Option<rodio::decoder::Decoder<std::fs::File>> {
    let f = fs::File::open(path).expect("Invalid sound source path!");
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
