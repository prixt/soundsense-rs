use super::*;

pub struct SoundChannel {
	pub looping: Sink,
	pub files: Vec<SoundFile>,
	pub one_shots: Vec<Sink>,
	pub volume: f32,
}
impl SoundChannel {
	pub fn new(device: &Device) -> Self {
		Self {
			looping : Sink::new(device),
			files : Vec::new(),
			one_shots : Vec::new(),
			volume : 1.0,
		}
	}

	pub fn maintain(&mut self, device: &Device, rng: &mut ThreadRng, _ui_handle: Option<&UIHandle>) {
		self.one_shots.retain(|s| !s.empty());
		if self.one_shots.is_empty() {
			if self.looping.empty() && !self.files.is_empty() {
				self.looping = Sink::new(device);
				for file in self.files.iter() {
					append_soundfile_to_sink(&self.looping, file, true, rng);
				}
			}
			self.looping.play();
		}
	}

	pub fn change_loop(&mut self, device: &Device, files: &[SoundFile], rng: &mut ThreadRng) {
		self.looping.stop();
		self.files.clear();
		self.files.extend_from_slice(files);
		self.maintain(device, rng, None);
	}

	pub fn add_oneshot(&mut self, device: &Device, file: &SoundFile, rng: &mut ThreadRng) {
		self.looping.pause();
		let sink = Sink::new(device);
		append_soundfile_to_sink(&sink, file, false, rng);
		self.one_shots.push(sink);
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

fn append_soundfile_to_sink(sink: &Sink, soundfile: &SoundFile, is_looping: bool, rng: &mut ThreadRng) {
	match soundfile.r#type {
		SoundFileType::IsPath(ref path) => {
			assert_file(path, sink);
		}
		SoundFileType::IsPlaylist(ref paths) => {
			if is_looping {
				paths.iter().for_each(|p| {
					assert_file(p, sink);
				});
			} else {
				assert_file(&paths.choose(rng).unwrap(), sink);
			}
		}
	}
}

fn assert_file(path: &Path, sink: &Sink) {
	let f = fs::File::open(path).unwrap();
	let source = Decoder::new(f);
	match source {
		Ok(source) => {
			sink.append(source.buffered().convert_samples::<f32>());
		},
		Err(e) => {
			println!("error: {}, path: {}", e, path.to_string_lossy());
		}
	}
}