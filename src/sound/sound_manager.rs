use super::*;

pub struct SoundManager {
    sounds: Vec<SoundEntry>,
    recent: HashSet<usize>,
    ignore_list: Vec<Regex>,
    device: Device,
    channels: HashMap<Box<str>, SoundChannel>,
    total_volume: f32,
    concurency: usize,
    // ui_sender: Sender<UIMessage>,
    rng: ThreadRng,
}

impl SoundManager {
	pub fn new(sound_dir: &Path, ui_sender: Sender<UIMessage>) -> Self {
		let mut sounds = Vec::new();
		let device = default_output_device().expect("Failed to get default audio output device.");
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
				} else if path.is_file() && path.extension().map_or(false, |ext| ext=="xml") {
					func(&path);
				}
			}
		}

        let mut func = |file_path: &Path| {
            use quick_xml::{Reader, events::Event};
            let mut reader = Reader::from_file(file_path).unwrap();

            let mut current_sound : Option<SoundEntry> = None;

            let buf = &mut Vec::new();
            loop {
                match reader.read_event(buf) {
                    Ok(Event::Start(ref data)) | Ok(Event::Empty(ref data)) => {
                        let local_name = data.local_name();
                        if local_name == b"sound" {

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

                            for attr in data.attributes().with_checks(false) {
                                let attr = attr.unwrap();
                                let attr_value = unsafe {std::str::from_utf8_unchecked(&attr.value)};
                                match attr.key {
                                    b"logPattern" => {
                                        let processed = FAULTY_ESCAPE.replace_all(&attr_value, "$1");
                                        let processed = EMPTY_EXPR.replace_all(&processed, ")?");
                                        pattern = Some(Regex::new(&processed).unwrap());
                                    }
                                    b"channel" => {
                                        let channel_name : Box<str> = attr_value.into();
                                        if !channels.contains_key(&channel_name) {
                                            channels.insert(channel_name.clone(), SoundChannel::new(&device));
                                        }
                                        channel = Some(channel_name);
                                    }
                                    b"loop" => {
                                        loop_attr.replace(attr_value == "start");
                                    }
                                    b"concurency" => {
                                        concurency = Some( attr_value.parse().unwrap() );
                                    }
                                    b"timeout" => {
                                        timeout = Some( attr_value.parse().unwrap() );
                                    }
                                    // Probability was mispelled...
                                    b"propability" => {
                                        probability = Some( attr_value.parse().unwrap() );
                                    }
                                    b"delay" => {
                                        delay = Some( attr_value.parse().unwrap() );
                                    }
                                    b"haltOnMatch" => {
                                        halt_on_match = attr_value == "true";
                                    }
                                    b"randomBalance" => {
                                        random_balance = attr_value == "true" ;
                                    }
                                    b"ansiFormat" => (),
                                    b"ansiPattern" => (),
                                    b"playbackThreshhold" => (),
                                    _ => {
                                        println!(
                                            "Unknown sound value: {}",
                                            unsafe {std::str::from_utf8_unchecked(attr.key)}
                                        );
                                    }
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

                        else if current_sound.is_some() && local_name == b"soundFile" {

                            let mut path = PathBuf::from(file_path);
                            path.pop();
                            let mut is_playlist = false;
                            let mut weight: f32 = 100.0;		
                            let mut volume: f32 = 1.0;	
                            let mut random_balance: bool = false;
                            let mut balance: f32 = 0.0;
                            let mut delay: usize = 0;

                            for attr in data.attributes() {
                                let attr = attr.unwrap();
                                let attr_value = unsafe {
                                    std::str::from_utf8_unchecked(&attr.value)
                                };
                                match attr.key {
                                    b"fileName" => path.push(attr_value),
                                    b"weight" => {
                                        weight = attr_value.parse().unwrap();
                                    }
                                    b"volumeAdjustment" => {
                                        // TODO: check if linear conversion from decibel to normal volume does work
                                        volume = (attr_value.parse::<f32>().unwrap() + 40.0) / 40.0;
                                    }
                                    b"randomBalance" => {
                                        random_balance = attr_value == "true";
                                    }
                                    b"balanceAdjustment" => {
                                        balance = attr_value.parse().unwrap();
                                    }
                                    b"delay" => {
                                        delay = attr_value.parse().unwrap();
                                    }
                                    b"playlist" => {
                                        is_playlist = true;
                                    }
                                    _ => {
                                        println!(
                                            "Unknown sound value: {}",
                                            unsafe {std::str::from_utf8_unchecked(attr.key)}
                                        );
                                    }
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

                    Ok(Event::End(data)) => {
                        if current_sound.is_some() && data.local_name() == b"sound" {
                            sounds.push( current_sound.take().unwrap() );
                        }
                    },

                    Ok(Event::Eof) => break,

                    Err(e) => panic!("Error parsing xml at position {}: {:?}", reader.buffer_position(), e),

                    _ => ()
                }
            }
        };

        visit_dir(sound_dir, &mut func);

        let mut channel_names: Vec<Box<str>> = vec![
            "all".into(),
            "music".into(),
            "weather".into(),
            "trade".into(),
            "swords".into(),
            "misc".into(),
        ];
        for channel_name in channels.keys() {
            if !channel_names.contains(channel_name) {
                channel_names.push(channel_name.clone());
            }
        }
        ui_sender.send(UIMessage::LoadedSoundpack(channel_names)).unwrap();

        // println!("Finished loading!");
        Self {
            sounds,
            recent: HashSet::new(),
            ignore_list: Vec::new(),
            device,
            channels,
            total_volume: 1.0,
            concurency: 0,
            // ui_sender,
            rng: thread_rng(),
        }
    }

	pub fn maintain(&mut self) {
		self.concurency = 0;
		{
			let sounds = &mut self.sounds;
			let recent = &mut self.recent;
			recent.retain(|&i| {
				let timeout = sounds[i].current_timeout.saturating_sub(100);
				let recent_call = sounds[i].recent_call.saturating_sub(1);
				sounds[i].current_timeout = timeout;
				sounds[i].recent_call = recent_call;
				timeout != 0
			});
		}
		for chn in self.channels.values_mut() {
			chn.maintain(&self.device, &mut self.rng);
			self.concurency += chn.len();
		}
	}

    pub fn set_volume(&mut self, channel_name: &str, volume: f32) {
        if channel_name == "all" {
            self.total_volume = volume;
            for channel in self.channels.values_mut() {
                channel.set_volume(channel.local_volume, self.total_volume);
            }
        }
        else if let Some(channel) = self.channels.get_mut(channel_name) {
            channel.set_volume(volume, self.total_volume);
        }
    }

    pub fn set_ignore_list(&mut self, ignore_list: Vec<Regex>) {
        std::mem::replace(&mut self.ignore_list, ignore_list);
    }

    pub fn process_log(&mut self, log: &str) {
        println!("log: {}", log);

        for pattern in self.ignore_list.iter() {
            if pattern.is_match(log) {
                return
            }
        }

        let rng = &mut self.rng;
        let sounds = &mut self.sounds;
        let recent = &mut self.recent;

        for (i, sound) in sounds.iter_mut().enumerate() {
            if sound.pattern.is_match(log) {
                println!("--pattern: {}", sound.pattern.as_str());
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
    let parent_path = path.parent().unwrap();

    let mut path_vec = Vec::new();
    let mut f = File::open(path).unwrap();
    let buf = &mut String::new();
    let extension = path.extension().unwrap();
    if extension == "m3u" {
        f.read_to_string(buf).unwrap();
        for line in buf.lines() {
            lazy_static! {
                static ref M3U_PATTERN: Regex = Regex::new(
                        r"#EXT[A-Z]*"
                    ).unwrap();
            }

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
            lazy_static! {
                static ref PLS_PATTERN: Regex = Regex::new(
                        r"File.+=(.+)"
                    ).unwrap();
            }
            
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