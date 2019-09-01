use super::*;
use regex::Regex;

pub struct SoundManager {
	sounds: Vec<SoundEntry>,
	recent: HashSet<usize>,
	device: Device,
	channels: HashMap<Box<str>, SoundChannel>,
	total_volume: f32,
	concurency: usize,
	ui_handle: UIHandle,
	rng: ThreadRng,
}

impl SoundManager {
	pub fn new(sound_dir: &Path, mut ui_handle: UIHandle) -> Self {
		use xml::reader::{EventReader, XmlEvent};

		let mut sounds = Vec::new();
		let device = default_output_device().unwrap();
		let mut channels : HashMap<Box<str>, SoundChannel> = HashMap::new();
		channels.insert(
			String::from("misc").into_boxed_str(),
			SoundChannel::new(&device)
		);

		fn visit_dir(dir: &Path, func: &mut dyn FnMut(&Path)) {
			for entry in fs::read_dir(dir).unwrap() {
				let entry = entry.unwrap();
				let path = entry.path();
				if path.is_dir() {
					visit_dir(&path, func);
				} else if path.is_file() && path.extension().unwrap() == "xml" {
					func(&path);
				}
			}
		}

		let mut func = |file_path: &Path| {
			let file = fs::File::open(file_path).unwrap();
			let file = io::BufReader::new(file);
			let parser = EventReader::new(file);

			let mut current_sound : Option<SoundEntry> = None;

			for e in parser {
				match e.unwrap() {
					XmlEvent::StartElement{name, attributes, ..} => {
						if name.local_name == "sound" {

							let mut pattern: Option<Regex> = None;
							let mut channel: Option<Box<str>> = None;
							let mut loop_attr: Option<bool> = None;
							let mut concurency: Option<usize> = None;
							let mut timeout: Option<usize> = None;
							let mut probability: Option<usize> = None;
							let mut delay: Option<usize> = None;
							let mut halt_on_match: bool = false;
							let mut random_balance: bool = false;
							let files = Vec::new();
							let weights = Vec::new();

							for attr in attributes {
								let attribute_name = attr.name.local_name.as_str();
								match attribute_name {
									"logPattern" => {
										lazy_static! {
											static ref FAULTY_ESCAPE: Regex = Regex::new(
												r"\\([^\.\+\*\?\(\)\|\[\]\{\}\^\$])"
											).unwrap();

											static ref EMPTY_EXPR: Regex = Regex::new(
												r"(\|\(\)\))"
											).unwrap();
										}
										let mut processed = attr.value;
										processed = FAULTY_ESCAPE.replace_all(&processed, "$1").into();
										processed = EMPTY_EXPR.replace_all(&processed, ")?").into();
										pattern = Some(Regex::new(&processed).unwrap());
									},
									"channel" => {
										let channel_name : Box<str> = attr.value.into();
										if !channels.contains_key(&channel_name) {
											channels.insert(channel_name.clone(), SoundChannel::new(&device));
										}
										channel = Some(channel_name);
									},
									"loop" => if attr.value == "start" {
										loop_attr.replace(true);
									}
									else {
										loop_attr.replace(false);
									},
									"concurency" => {
										concurency = Some( attr.value.parse().unwrap() );
									},
									"timeout" => {
										timeout = Some( attr.value.parse().unwrap() );
									},
									// Probability was mispelled...
									"propability" => {
										probability = Some( attr.value.parse().unwrap() );
									},
									"delay" => {
										delay = Some( attr.value.parse().unwrap() );
									},
									"haltOnMatch" => if attr.value == "true" {
										halt_on_match = true;
									},
									"randomBalance" => if attr.value == "true" {
										random_balance = true;
									}
									"ansiFormat" => (),
									"ansiPattern" => (),
									"playbackThreshhold" => (),
									_ => println!("Unknown sound value: {}", attribute_name)
								}
							}

							current_sound = Some(
								SoundEntry{
									pattern: pattern.unwrap(),
									channel,
									loop_attr,
									concurency,
									timeout,
									probability,
									delay,
									halt_on_match,
									random_balance,
									files,
									weights,
									current_timeout: 0,
									recent_call: 0,
								}
							);
						}

						else if current_sound.is_some() && name.local_name == "soundFile" {

							let mut path = PathBuf::from(file_path);
							path.pop();
							let mut is_playlist = false;
							let mut weight: f32 = 100.0;		
							let mut volume: f32 = 1.0;	
							let mut random_balance: bool = false;
							let mut balance: f32 = 0.0;
							let mut delay: usize = 0;

							for attr in attributes {
								let attr_name = attr.name.local_name.as_str();
								match attr_name {
									"fileName" => path.push(attr.value),
									"weight" => {
										weight = attr.value.parse().unwrap();
									}
									"volumeAdjustment" => {
										// TODO: check if linear conversion from decibel to normal volume does work
										volume = (attr.value.parse::<f32>().unwrap() + 40.0) / 40.0;
									}
									"randomBalance" => {
										if attr.value == "true" { 
											random_balance = true;
										}
									}
									"balanceAdjustment" => {
										balance = attr.value.parse().unwrap();
									}
									"delay" => {
										delay = attr.value.parse().unwrap();
									}
									"playlist" => {
										is_playlist = true;
									}
									_ => println!("Unknown sound-file value: {}", attr_name)
								}
							}
							let r#type = if is_playlist {
								let path_vec = parse_playlist(&path);
								SoundFileType::IsPlaylist(path_vec)
							} else {
								// test_file(&path);
								SoundFileType::IsPath(path)
							};
							let sound_file = SoundFile {
								r#type,
								weight,
								volume,
								random_balance,
								delay,
								balance,
							};
							let sound = current_sound.as_mut().unwrap();
							sound.files.push(sound_file);
							sound.weights.push(weight);
						}
					},

					XmlEvent::EndElement{name} => {
						if current_sound.is_some() && name.local_name == "sound" {
							sounds.push( current_sound.take().unwrap() );
						}
					},

					_ => ()
				}
			}
		};

		visit_dir(sound_dir, &mut func);
		ui_handle.clear_sliders();
		ui_handle.add_slider("all".to_string());
		ui_handle.add_slider("music".to_string());
		ui_handle.add_slider("weather".to_string());
		ui_handle.add_slider("trade".to_string());
		ui_handle.add_slider("misc".to_string());
		for channel in channels.keys() {
			ui_handle.add_slider(channel.to_string());
		}

		// println!("Finished loading!");
		Self {
			sounds,
			recent: HashSet::new(),
			device,
			channels,
			total_volume: 1.0,
			concurency: 0,
			ui_handle,
			rng: thread_rng(),
		}
	}

	pub fn maintain(&mut self) {
		self.concurency = 0;
		{
			let sounds = &mut self.sounds;
			let recent = &mut self.recent;
			recent.retain(|&i| {
				let timeout = sounds[i].current_timeout.checked_sub(100).unwrap_or(0);
				let recent_call = sounds[i].recent_call.checked_sub(1).unwrap_or(0);
				sounds[i].current_timeout = timeout;
				sounds[i].recent_call = recent_call;
				timeout != 0
			});
		}
		for chn in self.channels.values_mut() {
			chn.maintain(&self.device, &mut self.rng, Some(&self.ui_handle));
			self.concurency += chn.len();
		}
	}

	pub fn set_volume(&mut self, channel_name: &str, volume: f32) {
		if channel_name == "all" {
			self.total_volume = volume;
			for channel in self.channels.values_mut() {
				channel.set_volume(channel.volume, self.total_volume);
			}
		}
		else if let Some(channel) = self.channels.get_mut(channel_name) {
			channel.set_volume(volume, self.total_volume);
		}
	}

	pub fn process_log(&mut self, log: &str) {
		// println!("log: {}", log);

		let rng = &mut self.rng;

		let sounds = &mut self.sounds;
		let recent = &mut self.recent;

		for (i, sound) in sounds.iter_mut().enumerate() {
			if sound.pattern.is_match(log) {
				// println!("--pattern: {}", sound.pattern.as_str());
				recent.insert(i);
				sound.recent_call += 1;

				let mut can_play = sound.current_timeout == 0;
				if can_play {
					if let Some(probability) = sound.probability {
						can_play &= probability >= rng.next_u32() as usize;
					}
					if let Some(concurency) = sound.concurency {
						can_play &= self.concurency <= concurency;
					}
				}

				if can_play {
					let files = &sound.files;
					let idx : usize = if files.len() > 1 && !sound.loop_attr.unwrap_or(false) {
						WeightedIndex::new(&sound.weights).unwrap().sample(rng)
					} else {
						0
					};
					if let Some(timeout) = sound.timeout {
						sound.current_timeout = timeout;
					}
					// Prevent repeated alerts from firing constantly.
					if sound.recent_call >= sound.current_timeout + 5 {
						sound.current_timeout = sound.recent_call * 100;
					}
					if let Some(chn) = &sound.channel {
						println!("--channel: {}", chn);
						let device = &self.device;
						let channel = self.channels.get_mut(chn).unwrap();
						
						if let Some(is_loop_start) = sound.loop_attr {
							if is_loop_start {
								println!("----loop=start");
								channel.change_loop(device, sound.files.as_slice(), sound.delay.unwrap_or(0), rng);
							} else {
								println!("----loop=stop");
								channel.change_loop(device, &[], sound.delay.unwrap_or(0), rng);
								if !sound.files.is_empty() {
									channel.add_oneshot(device, &files[idx], sound.delay.unwrap_or(0), rng);
								}
							}
						}
						else if !sound.files.is_empty() && channel.len() <= sound.concurency.unwrap_or(std::usize::MAX) {
							channel.add_oneshot(device, &files[idx], sound.delay.unwrap_or(0), rng);
						}
					
					} else if !sound.files.is_empty() {
						let channel = self.channels.get_mut("misc").unwrap();
						if channel.len() <= sound.concurency.unwrap_or(std::usize::MAX) {
							channel.add_oneshot(&self.device, &files[idx], sound.delay.unwrap_or(0), rng);
						}
					}
				}

				if sound.halt_on_match {
					break;
				}
			}
		}
	}
}

fn parse_playlist(path: &Path) -> Vec<PathBuf> {
	lazy_static! {
		static ref M3U_PATTERN: Regex = Regex::new(
				r"#EXT[A-Z]*"
			).unwrap();
		static ref PLS_PATTERN: Regex = Regex::new(
				r"File.+=(.+)"
			).unwrap();
	}

	let parent_path = path.parent().unwrap();

	let mut path_vec = Vec::new();
	let mut f = File::open(path).unwrap();
	let buf = &mut String::new();
	let extension = path.extension().unwrap();
	if extension == "m3u" {
		f.read_to_string(buf).unwrap();
		for line in buf.lines() {
			if !M3U_PATTERN.is_match(line) {
				let mut path = PathBuf::from(parent_path);
				path.push(line);
				path_vec.push(path);
			}
		}
	}
	else if extension == "pls" {
		f.read_to_string(buf).unwrap();
		for line in buf.lines() {
			if let Some(caps) = PLS_PATTERN.captures(line) {
				let mut path = PathBuf::from(parent_path);
				path.push(&caps[0]);
				path_vec.push(path);
			}
		}
	}
	else {
		unreachable!("Playlist {:?} is not valid!", path)
	}
	
	path_vec
}