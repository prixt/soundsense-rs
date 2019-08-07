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

	pub fn maintain(&mut self, device: &Device, _ui_sender: Option<&glib::Sender<UIMessage>>) {
		self.one_shots.retain(|s| !s.empty());
		if self.one_shots.is_empty() {
			if self.looping.empty() && !self.files.is_empty() {
				self.looping = Sink::new(device);
				for file in self.files.iter() {
					let f = fs::File::open(&file.path).unwrap();
					let source = Decoder::new(f).unwrap()
						.buffered().convert_samples::<f32>();
					self.looping.append(source);
				}
			}
			self.looping.play();
		} else {
			self.looping.pause();
		}
	}

	pub fn change_loop(&mut self, device: &Device, files: &[SoundFile]) {
		self.looping.stop();
		self.files.clear();
		self.files.extend_from_slice(files);
		self.maintain(device, None);
	}

	pub fn add_oneshot(&mut self, device: &Device, file: &SoundFile) {
		let f = fs::File::open(&file.path).unwrap();
		let source = Decoder::new(f).unwrap();
		let sink = Sink::new(device);
		sink.append(source);
		self.one_shots.push(sink);
		self.looping.pause();
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