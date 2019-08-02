use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use rodio::*;
use rand::prelude::*;

pub struct SoundManager {
	sounds: Vec<Sound>,
	device: Device,
	channels: HashMap<String, Channel>,
	rng: ThreadRng,
}

struct Sound {
	pattern: regex::Regex,		 // regular expression matching log line
	channel: Option<String>,	 // channel on which sound is played. sounds played on channel can be looped/stopped prematurely
	loop_attr: Option<bool>,	 // "start" - sound start loop on channel until different sound is played on channel (if it is non-looped sound, loop will resume when it is done playing) or sound with "stop" is triggered.
	concurency: Option<u32>,	 // number of councured sounds allowd to be played besides this sound. If currenty playing more than that, sound is ignored. In miliseconds, default unlimited.
	timeout: Option<u32>,		 // number, timeout during which is sound going to be prevented from playing again. In miliseconds default 0.
	probability: Option<u32>,	 // percentage, Propablity that sound will be played. Default is always played.
	delay: Option<u32>,			 // number, delay before sound is played. In miliseconds, default 0.
	halt_on_match: bool,		 // boolean, if set to true, sound sense will stop processing long line after it was matched to this sound. Default false
	files: Vec<SoundFile>,
}

#[derive(Clone)]
struct SoundFile {
	path: PathBuf,			// path to audio file with sound.
	weight: u32,			// controls likelihood of sound to be chosen. Default is 100.
	volume: f32,			// adjusts volume of sample. Can range from -40 to +6 decibles, default 0.
	random_balance: bool,	// if set to true will randomply distribute sound between stereo channels.
	balance: f32,			// adjusts stereo channel, can range for -1 (full left) to 1 (full right).
}

struct Channel {
	looping: Sink,
	files: Vec<SoundFile>,
	one_shots: Vec<Sink>,
	volume: f32,
}
impl Channel {
	fn new(device: &Device) -> Self {
		Self {
			looping : Sink::new(device),
			files : Vec::new(),
			one_shots : Vec::new(),
			volume : 1.0,
		}
	}

	fn maintain(&mut self, device: &Device) {
		let mut play_loop = true;
		for s in self.one_shots.iter() {
			play_loop &= s.empty();
		}
		if play_loop {
			if self.looping.empty() && !self.files.is_empty() {
				self.looping = Sink::new(device);
				for file in self.files.iter() {
					let f = fs::File::open(&file.path).unwrap();
					let source = Decoder::new(f).unwrap()
						.buffered().convert_samples::<f32>();
					self.looping.append(source);
				}
				self.looping.play();
			}
			self.one_shots.clear();
		} else {
			self.looping.pause();
		}
	}

	fn change_loop(&mut self, device: &Device, files: &[SoundFile], rng: &mut ThreadRng) {
		self.looping.stop();
		self.files.clear();
		self.files.extend_from_slice(files);
		self.files.shuffle(rng);
		self.maintain(device);
	}

	fn add_oneshot(&mut self, device: &Device, file: &SoundFile) {
		let f = fs::File::open(&file.path).unwrap();
		let source = Decoder::new(f).unwrap();
		let sink = Sink::new(device);
		sink.append(source);
		self.one_shots.push(sink);
		self.looping.pause();
	}

	fn set_volume(&mut self, volume: f32) {
		self.volume = volume;
		self.looping.set_volume(volume);
		for s in self.one_shots.iter() {
			s.set_volume(volume);
		}
	}

	fn len(&self) -> u32 {
		let mut length = self.looping.len();
		for s in self.one_shots.iter() {
			if !s.empty() {
				length += 1;
			}
		}
		length as u32
	}
}

impl SoundManager {
	pub fn new(sound_dir: &Path) -> Self {
		use xml::reader::{EventReader, XmlEvent};

		let mut sounds = Vec::new();
		let device = default_output_device().unwrap();
		let channels = HashMap::new();

		fn visit_dir(dir: &Path, func: &mut FnMut(&Path)) {
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

			let mut current_sound : Option<Sound> = None;

			for e in parser {
				match e.unwrap() {
					XmlEvent::StartElement{name, attributes, ..} => {
						if name.local_name == "sound" {

							let mut pattern = String::new();
							let mut channel: Option<String> = None;
							let mut loop_attr: Option<bool> = None;
							let mut concurency: Option<u32> = None;
							let mut timeout: Option<u32> = None;
							let mut probability: Option<u32> = None;
							let mut delay: Option<u32> = None;
							let mut halt_on_match: bool = false;
							let files = Vec::new();

							for attr in attributes {
								let attr_name = &attr.name.local_name;
								if attr_name == "logPattern" {
									pattern.clone_from(&attr.value);
								}
								else if attr_name == "channel" {
									channel.replace(attr.value.clone());
								}
								else if attr_name == "loop" {
									if attr.value == "start" {
										loop_attr.replace(true);
									}
									else {
										loop_attr.replace(false);
									}
								}
								else if attr_name == "concurency" {
									concurency.replace( attr.value.parse().unwrap() );
								}
								else if attr_name == "timeout" {
									timeout.replace( attr.value.parse().unwrap() );
								}
								else if attr_name == "probability" {
									probability.replace( attr.value.parse().unwrap() );
								}
								else if attr_name == "delay" {
									delay.replace( attr.value.parse().unwrap() );
								}
								else if attr_name == "haltOnMatch"
									&& attr.value == "true" {
									halt_on_match = true;
								}
							}

							let pattern = regex::Regex::new(&pattern).unwrap();
							current_sound = Some(
								Sound{
									pattern,
									channel,
									loop_attr,
									concurency,
									timeout,
									probability,
									delay,
									halt_on_match,
									files
								}
							);
						}

						else if name.local_name == "soundFile" {

							let mut path = PathBuf::from(file_path);
							path.pop();
							let mut weight: u32 = 0;		
							let mut volume: f32 = 0.0;	
							let mut random_balance: bool = false;
							let mut balance: f32 = 0.0;

							for attr in attributes {
								let attr_name = &attr.name.local_name;
								if attr_name == "fileName" {
									path.push(attr.value);
								}
								else if attr_name == "weight" {
									weight = attr.value.parse().unwrap();
								}
								else if attr_name == "volumeAdjustment" {
									volume = attr.value.parse().unwrap();
								}
								else if attr_name == "randomBalance" && attr.value == "true" {
									random_balance = true;
								}
								else if attr_name == "balanceAdjustment" {
									balance = attr.value.parse().unwrap();
								}
							}
							let sound_file = SoundFile {
								path,
								weight,
								volume,
								random_balance,
								balance,
							};
							current_sound.as_mut().unwrap()
								.files.push(sound_file);
						}
					},

					XmlEvent::EndElement{name} => {
						if name.local_name == "sound" {
							sounds.push( current_sound.take().unwrap() );
						}
					},

					_ => ()
				}
			}
		};

		visit_dir(sound_dir, &mut func);

		println!("Finished loading!");
		Self {
			sounds,
			device,
			channels,
			rng: thread_rng()
		}
	}

	pub fn maintain(&mut self, _sender: &mut std::sync::mpsc::Sender<(String, String)>) {
		for (key, chn) in self.channels.iter_mut() {
			chn.maintain(&self.device);
		}
	}

	pub fn process_log(&mut self, log: &str) {
		for sound in self.sounds.iter() {
			if sound.pattern.is_match(log) {
				println!("log: {}", log);
				println!("pattern: {}", sound.pattern.as_str());
				if let Some(chn) = &sound.channel {
					println!("channel: {}", chn);
					let device = &self.device;
					let channel = self.channels.entry(chn.clone())
						.or_insert_with(|| Channel::new(&device));
					
					if let Some(is_loop_start) = sound.loop_attr {
						if is_loop_start {
							channel.change_loop(device, sound.files.as_slice(), &mut self.rng);
						} else {
							channel.change_loop(device, &[], &mut self.rng);
							if !sound.files.is_empty() {
								channel.add_oneshot(device, (&sound.files).choose(&mut self.rng).unwrap());
							}
						}
					}
					else if !sound.files.is_empty() && channel.len() <= sound.concurency.or(Some(std::u32::MAX)).unwrap() {
						channel.add_oneshot(device, (&sound.files).choose(&mut self.rng).unwrap());
					}
				
				} else if !sound.files.is_empty() {
					let sound_file = (&sound.files).choose(&mut self.rng).unwrap();
					println!("playing: {}", sound_file.path.to_str().unwrap());
					let f = fs::File::open(&sound_file.path).unwrap();
					let source = Decoder::new(f).unwrap().buffered()
						.convert_samples();
					play_raw(&self.device, source);
				}

				if sound.halt_on_match {
					break;
				}
			}
		}
	}
}