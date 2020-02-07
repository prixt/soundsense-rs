use super::*;

pub struct SoundChannel {
    pub looping: SpatialSink,
    pub files: Vec<SoundFile>,
    pub one_shots: Vec<SpatialSink>,
    pub volume: f32,
    pub delay: usize,
}

impl SoundChannel {
    pub fn new(device: &Device) -> Self {
        Self {
            looping : SpatialSink::new(device, [0.0, 0.0, 0.0], [-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
            files : Vec::new(),
            one_shots : Vec::new(),
            volume : 1.0,
            delay : 0,
        }
    }

	pub fn maintain(&mut self, device: &Device, rng: &mut ThreadRng, _ui_handle: Option<&UIHandle>) {
		let delay = self.delay.saturating_sub(100);
		self.delay = delay;
		self.one_shots.retain(|s| {
			if delay != 0 {
				s.pause();
			} else {
				s.play();
			}
			!s.empty()
		});
		self.looping.play();
		if self.one_shots.is_empty() && delay == 0 {
			if self.looping.empty() && !self.files.is_empty() {
				self.looping = SpatialSink::new(device, [0.0, 0.0, 0.0], [-2.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
				for file in self.files.iter() {
					append_soundfile_to_sink(&self.looping, file, true, rng);
				}
			}
		} else {
			self.looping.pause();
		}
	}

    pub fn change_loop(&mut self, device: &Device, files: &[SoundFile], delay: usize, rng: &mut ThreadRng) {
        self.looping.stop();
        self.files.clear();
        self.files.extend_from_slice(files);
        self.delay = delay;
        self.maintain(device, rng, None);
    }

    pub fn add_oneshot(&mut self, device: &Device, file: &SoundFile, delay: usize, rng: &mut ThreadRng) {
        self.looping.pause();
        let sink = SpatialSink::new(device, [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        append_soundfile_to_sink(&sink, file, false, rng);
        self.one_shots.push(sink);
        self.delay = delay;
    }

    pub fn set_volume(&mut self, local_volume: f32, total_volume: f32) {
        self.volume = local_volume;
        let final_volume = local_volume * total_volume;
        self.looping.set_volume(final_volume);
        self.one_shots.iter()
            .for_each(|s| s.set_volume(final_volume));
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.one_shots.len() + !(self.files.is_empty() || self.looping.is_paused()) as usize
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
            println!("error: {}, path: {}", e, path.to_string_lossy());
        }
    }
}