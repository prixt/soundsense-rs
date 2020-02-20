use super::*;

pub struct SoundManager {
    sounds: Vec<SoundEntry>,
    recent: HashSet<usize>,
    ignore_list: Vec<Regex>,
    device: Device,
    channels: BTreeMap<Box<str>, SoundChannel>,
    total_volume: VolumeLock,
    ui_sender: Sender<UIMessage>,
    rng: ThreadRng,
}

impl SoundManager {
	pub fn new(sound_dir: &Path, ui_sender: Sender<UIMessage>) -> Result<Self> {
        let total_volume = VolumeLock::new();
		let mut sounds = Vec::new();
        let device = default_output_device()
            .ok_or("Failed to get default audio output device.")?;
		let mut channels : BTreeMap<Box<str>, SoundChannel> = BTreeMap::new();
		channels.insert(
			String::from("misc").into_boxed_str(),
			SoundChannel::new(&device, "misc", total_volume.clone())
		);

		fn visit_dir(dir: &Path, func: &mut dyn FnMut(&Path)->Result<()>) -> Result<()> {
            println!("Directory: {:?}", dir);
            match fs::read_dir(dir) {
                Ok(entries) => for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, func)?;
                    } else if path.is_file() && path.extension().map_or(false, |ext| ext=="xml") {
                        func(&path)?;
                    }
                },
                Err(e) => eprintln!("Error while visiting {}: {}", dir.display(), e),
            }
            Ok(())
		}

        let mut func = |file_path: &Path| -> Result<()> {
            use quick_xml::{Reader, events::Event};
            println!("-File: {:?}", file_path);
            let mut reader = Reader::from_file(file_path)?;

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
                            #[allow(unused_mut)]
                            let mut playback_threshold: u8 = 5;
                            let files = Vec::new();
                            let weights = Vec::new();

                            for attr in data.attributes().with_checks(false) {
                                let attr = attr?;
                                let attr_value = unsafe {std::str::from_utf8_unchecked(&attr.value)};
                                match attr.key {
                                    b"logPattern" => {
                                        let processed = FAULTY_ESCAPE.replace_all(&attr_value, "$1");
                                        let processed = EMPTY_EXPR.replace_all(&processed, ")?");
                                        pattern = Some(Regex::new(&processed)?);
                                    }
                                    b"channel" => {
                                        let channel_name : Box<str> = attr_value.into();
                                        if !channels.contains_key(&channel_name) {
                                            channels.insert(channel_name.clone(), SoundChannel::new(&device, &channel_name, total_volume.clone()));
                                        }
                                        channel = Some(channel_name);
                                    }
                                    b"loop" => {
                                        loop_attr.replace(attr_value == "start");
                                    }
                                    b"concurency" => {
                                        concurency = Some( attr_value.parse()? );
                                    }
                                    b"timeout" => {
                                        timeout = Some( attr_value.parse()? );
                                    }
                                    // Probability was mispelled...
                                    b"propability" | b"probability" => {
                                        probability = Some( attr_value.parse()? );
                                    }
                                    b"delay" => {
                                        delay = Some( attr_value.parse()? );
                                    }
                                    b"haltOnMatch" => {
                                        halt_on_match = attr_value == "true";
                                    }
                                    b"randomBalance" => {
                                        random_balance = attr_value == "true" ;
                                    }
                                    b"playbackThreshhold" => {
                                        playback_threshold = attr_value.parse()?;
                                    }
                                    b"ansiFormat" => (),
                                    b"ansiPattern" => (),
                                    _ => {
                                        eprintln!(
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
                                    playback_threshold,
                                    files,
                                    weights,
                                    current_timeout: 0,
                                    recent_call: 0,
                                }
                            );
                        }

                        else if local_name == b"soundFile" {
                            assert!(current_sound.is_some(), "SoundFile must be associated with a Sound!");
                            let mut path = PathBuf::from(file_path);
                            path.pop();
                            let mut is_playlist = false;
                            let mut weight: f32 = 100.0;		
                            let mut volume: f32 = 1.0;	
                            let mut random_balance: bool = false;
                            let mut balance: f32 = 0.0;
                            let mut delay: usize = 0;

                            for attr in data.attributes() {
                                let attr = attr?;
                                let attr_value = unsafe {std::str::from_utf8_unchecked(&attr.value)};
                                match attr.key {
                                    b"fileName" => path.push(attr_value),
                                    b"weight" => {
                                        weight = attr_value.parse()?;
                                    }
                                    b"volumeAdjustment" => {
                                        // TODO: check if linear conversion from decibel to normal volume does work
                                        volume = (attr_value.parse::<f32>()? + 40.0) / 40.0;
                                    }
                                    b"randomBalance" => {
                                        random_balance = attr_value == "true";
                                    }
                                    b"balanceAdjustment" => {
                                        balance = attr_value.parse()?;
                                    }
                                    b"delay" => {
                                        delay = attr_value.parse()?;
                                    }
                                    b"playlist" => {
                                        is_playlist = true;
                                    }
                                    _ => {
                                        eprintln!(
                                            "Unknown sound value: {}",
                                            unsafe {std::str::from_utf8_unchecked(attr.key)}
                                        );
                                    }
                                }
                            }
                            let r#type = if is_playlist {
                                let path_vec = parse_playlist(&path)?;
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
                            sounds.push( current_sound.take()
                                .ok_or("Tried to finish a Sound, even though there is no Sound!")?
                            );
                        }
                    },

                    Ok(Event::Eof) => return Ok(()),

                    Err(e) => panic!("Error parsing xml at position {}: {:?}", reader.buffer_position(), e),

                    _ => ()
                }
            }
        };

        visit_dir(sound_dir, &mut func)?;

        let mut channel_names: Vec<Box<str>> = vec![
            "all".into(),
            "music".into(),
        ];
        for channel_name in channels.keys() {
            if !channel_names.contains(channel_name) && channel_name.as_ref() != "misc" {
                channel_names.push(channel_name.clone());
            }
        }
        channel_names.push("misc".into());
        ui_sender.send(UIMessage::LoadedSoundpack(channel_names))?;

        println!("Soundpack loaded!");
        let mut manager = Self {
            sounds,
            recent: HashSet::new(),
            ignore_list: Vec::new(),
            device,
            channels,
            total_volume: total_volume.clone(),
            ui_sender,
            rng: thread_rng(),
        };

        let mut conf_path = dirs::config_dir().ok_or("No configuration directory found!")?;
        conf_path.push("soundsense-rs/default-volumes.ini");
        if conf_path.is_file() {
            let file = fs::File::open(conf_path)?;
            manager.get_default_volume(file)?;
        }

        Ok(manager)
    }

	pub fn maintain(&mut self, dt: usize) -> Result<()> {
		{
			let sounds = &mut self.sounds;
			let recent = &mut self.recent;
			recent.retain(|&i| {
				let timeout = sounds[i].current_timeout.saturating_sub(dt);
				let recent_call = sounds[i].recent_call.saturating_sub(1);
				sounds[i].current_timeout = timeout;
				sounds[i].recent_call = recent_call;
				timeout != 0
			});
		}
		for chn in self.channels.values_mut() {
			chn.maintain(&mut self.rng, dt);
		}
        Ok(())
	}

    pub fn set_volume(&mut self, channel_name: &str, volume: f32) -> Result<()> {
        if channel_name == "all" {
            self.total_volume.set(volume);
        }
        else if let Some(channel) = self.channels.get_mut(channel_name) {
            channel.set_local_volume(volume);
        }
        Ok(())
    }

    pub fn set_ignore_list(&mut self, ignore_list: Vec<Regex>) -> Result<()> {
        self.ignore_list = ignore_list;
        self.ui_sender.send(UIMessage::LoadedIgnoreList)?;
        Ok(())
    }

    pub fn process_log(&mut self, log: &str) -> Result<()> {
        println!("log: {}", log);

        for pattern in self.ignore_list.iter() {
            if pattern.is_match(log) {
                return Ok(())
            }
        }

        let rng = &mut self.rng;
        let sounds = &mut self.sounds;
        let recent = &mut self.recent;

        for (i, sound) in sounds.iter_mut().enumerate() {
            if sound.pattern.is_match(log) {
                println!("-pattern: {}", sound.pattern.as_str());
                recent.insert(i);
                sound.recent_call += 1;

                let mut can_play = sound.current_timeout == 0;
                if can_play {
                    if let Some(probability) = sound.probability {
                        can_play &= probability >= rng.next_u32() as usize;
                        if !can_play {
                            println!("--can't play: failed probability roll");
                        }
                    }
                } else {
                    println!("--can't play: current_timeout: {}", sound.current_timeout);
                }

                if can_play {
                    let files = &sound.files;
                    let idx : usize = if files.len() > 1 && !sound.loop_attr.unwrap_or(false) {
                        match WeightedIndex::new(&sound.weights) {
                            Ok(weight) => weight.sample(rng),
                            Err(e) => {
                                eprintln!("Error while weighing files: {}", e);
                                0
                            }
                        }
                    } else {
                        0
                    };

                    if let Some(chn) = &sound.channel {
                        print!("--channel: {}", chn);
                        let channel = if let Some(channel) = self.channels.get_mut(chn) {
                            channel
                        } else {
                            println!(" --doesn't exist in current soundpack!");
                            continue;
                        };
                        let chn_len = channel.len();
                        if chn_len < sound.concurency.unwrap_or(std::usize::MAX) {
                            if let Some(timeout) = sound.timeout {
                                sound.current_timeout = timeout;
                            }
                            let device = &self.device;
                            
                            if let Some(is_loop_start) = sound.loop_attr {
                                if is_loop_start {
                                    print!(" --loop=start");
                                    channel.change_loop(device, sound.files.as_slice(), sound.delay.unwrap_or(0), rng);
                                } else {
                                    print!(" --loop=stop");
                                    channel.stop_loop(sound.delay.unwrap_or(0));
                                    if !sound.files.is_empty() {
                                        channel.add_oneshot(device, &files[idx], sound.delay.unwrap_or(0), rng);
                                    }
                                }
                            }
                            else if !sound.files.is_empty() && channel.len() <= sound.concurency.unwrap_or(std::usize::MAX) {
                                channel.add_oneshot(device, &files[idx], sound.delay.unwrap_or(0), rng);
                            }
                            println!();
                        }
                        else {
                            println!(" --can't play: at concurency limit: limit {}, channel {}",
                                sound.concurency.unwrap(), chn_len);
                        }
                    }
                    else if !sound.files.is_empty() {
                        print!("--channel: misc");
                        let channel = self.channels.get_mut("misc").unwrap();
                        let chn_len = channel.len();
                        if channel.len() < sound.concurency.unwrap_or(std::usize::MAX) {
                            if let Some(timeout) = sound.timeout {
                                sound.current_timeout = timeout;
                            }
                            channel.add_oneshot(&self.device, &files[idx], sound.delay.unwrap_or(0), rng);
                        }
                        else {
                            println!(" --can't play: at concurency limit: limit {}, channel {}",
                                sound.concurency.unwrap(), chn_len);
                        }
                        println!();
                    }
                }

                if sound.halt_on_match {
                    return Ok(())
                }
            }
        }
        Ok(())
    }

    pub fn set_current_volumes_as_default(&self, mut file: File) -> Result<()> {
        use std::io::Write;
        writeln!(&mut file, "all={}", (self.total_volume.get()*100.0) as u32)?;
        for (channel_name, channel) in self.channels.iter() {
            writeln!(&mut file, "{}={}", channel_name, (channel.get_local_volume()*100.0) as u32)?;
        }
        Ok(())
    }

    fn get_default_volume(&mut self, mut file: File) -> Result<()> {
        lazy_static! {
            static ref INI_ENTRY: Regex = Regex::new("([[:word:]]+)=(.+)").unwrap();
        }
        let mut buf = String::new();
        let mut entries = vec![];
        file.read_to_string(&mut buf)?;
        for line in buf.lines() {
            if let Some(cap) =  INI_ENTRY.captures(line) {
                let name = cap.get(1)
                    .ok_or("Failed to parse .ini file.")?
                    .as_str();
                let volume: f32 = cap.get(2)
                    .ok_or("Failed to parse .ini file.")?
                    .as_str()
                    .parse()?;
                if name == "all" {
                    self.total_volume.set(volume / 100.0);
                }
                else if let Some(chn) = self.channels.get_mut(name) {
                    chn.set_local_volume(volume / 100.0);
                }
                entries.push((name.to_string().into_boxed_str(), volume));
            }
        }
        self.ui_sender
            .send(UIMessage::LoadedVolumeSettings(entries))?;
        Ok(())
    }
}

fn parse_playlist(path: &Path) -> Result<Vec<PathBuf>> {
    let parent_path = path.parent()
        .ok_or_else(||
            format!("Playlist {:?} doesn't have a parent directory!", path)
        )?;

    let mut path_vec = Vec::new();
    let mut f = File::open(path)?;
    let buf = &mut String::new();
    let extension = path.extension()
        .filter(|ext| *ext=="m3u" || *ext=="pls")
        .ok_or_else(|| format!(
            "Playlist {:?} is not valid! Playlist needs to have either .m3u or .pls extension.",
            path
        ))?;
    if extension == "m3u" {
        f.read_to_string(buf)?;
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
        f.read_to_string(buf)?;
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
    Ok(path_vec)
}