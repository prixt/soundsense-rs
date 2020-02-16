use super::*;

mod loop_player;
mod oneshot_player;

use loop_player::LoopPlayer;

pub struct SoundChannel {
    looping: LoopPlayer,
    one_shots: Vec<SpatialSink>,
    local_volume: f32,
    total_volume: f32,
    delay: usize,
    only_one_sound: bool,
}

impl SoundChannel {
    #[inline]
    pub fn new(device: &Device, name: &str) -> Self {
        Self {
            looping : LoopPlayer::new(device),
            one_shots : Vec::new(),
            local_volume : 1.0,
            total_volume : 1.0,
            delay : 0,
            only_one_sound: name == "weather" || name == "music",
        }
    }

	pub fn maintain(&mut self, rng: &mut ThreadRng, dt: usize) {
		let delay = self.delay.saturating_sub(dt);
		self.delay = delay;
		self.one_shots.retain(|s| {
			if delay != 0 {
				s.pause();
			} else {
				s.play();
			}
			!s.empty()
		});
		if self.one_shots.is_empty() && delay == 0 {
            self.looping.play();
            self.looping.maintain(rng);
		} else {
			self.looping.pause();
		}
	}

    pub fn change_loop(&mut self, device: &Device, files: &[SoundFile], delay: usize, rng: &mut ThreadRng) {
        self.looping.change_loop(device, files, rng);
        self.delay = delay;
        self.maintain(rng, 0);
        if self.only_one_sound {
            self.one_shots
                .drain(..)
                .for_each(|s| s.stop());
        }
    }

    pub fn stop_loop(&mut self, delay: usize) {
        self.looping.stop();
        self.delay = delay;
    }

    pub fn add_oneshot(&mut self, device: &Device, file: &SoundFile, delay: usize, rng: &mut ThreadRng) {
        self.looping.pause();
        let sink = SpatialSink::new(device, [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        append_soundfile_to_sink(&sink, file, false, rng);
        sink.set_volume(self.local_volume * self.total_volume);
        if self.only_one_sound {
            self.one_shots
                .drain(..)
                .for_each(|s| s.stop());
        }
        self.one_shots.push(sink);
        self.delay = delay;
    }

    #[inline]
    pub fn set_local_volume(&mut self, local_volume: f32) {
        self.local_volume = local_volume;
        self.set_final_volume(local_volume * self.total_volume);
    }
    #[inline]
    pub fn set_total_volume(&mut self, total_volume: f32) {
        self.total_volume = total_volume;
        self.set_final_volume(self.local_volume * total_volume);
    }
    #[inline]
    pub fn get_local_volume(&self) -> f32 {self.local_volume}

    #[inline]
    fn set_final_volume(&mut self, final_volume: f32) {
        self.looping.set_volume(final_volume);
        self.one_shots.iter()
            .for_each(|s| s.set_volume(final_volume));
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.one_shots.len() + !self.looping.is_paused() as usize
    }
}

fn append_soundfile_to_sink(sink: &SpatialSink, soundfile: &SoundFile, is_looping: bool, rng: &mut ThreadRng) {
    let volume = soundfile.volume;
    let balance: f32 = if soundfile.random_balance {
            rng.gen_range(-1.0, 1.0)
        } else {
            soundfile.balance
        };
    match soundfile.r#type {
        SoundFileType::IsPath(ref path) => {
            assert_file(path, sink, volume, balance);
        }
        SoundFileType::IsPlaylist(ref paths) => {
            if is_looping {
                paths.iter().for_each(|p| {
                    assert_file(p, sink, volume, balance);
                });
            } else {
                assert_file(&paths.choose(rng).unwrap(), sink, volume, balance);
            }
        }
    }
}

fn assert_file(path: &Path, sink: &SpatialSink, volume: f32, balance: f32) {
    let f = fs::File::open(path).unwrap();
    let source = Decoder::new(f);
    match source {
        Ok(source) => {
            let source = source.amplify(volume);
            sink.append(source.buffered().convert_samples::<f32>());
            sink.set_emitter_position([balance, 1.0, 0.0]);
        },
        Err(e) => {
            eprintln!("Error while asserting {}: {}", path.display(), e);
        }
    }
}
