use super::*;
use std::collections::HashMap;

/// The struct that parses the log entries.
/// Plays appropriate sounds on appropriate channels;
/// checks for concurrency, delays, and probability;
/// Sends messages to the UI after loading soundpack and ignore list.
pub struct SoundManager {
    /// All the Sounds loaded from the soundpack.
    sounds: Vec<SoundEntry>,
    /// The indices of the recently played Sounds.
    recent: HashSet<usize>,
    /// The previous log message. Replaces `x[0-9]+` messages. 
    previous_log: String,
    /// The patterns that SoundManager shouldn't process.
    ignore_list: Vec<Regex>,
    /// The sound device of the system.
    device: Device,
    /// All the channels, sorted alphabetically.
    channels: BTreeMap<Box<str>, SoundChannel>,
    /// The total volume.
    total_volume: VolumeLock,
    /// Total is_paused.
    total_is_paused: IsPausedLock,
    /// Total playback_treshold
    total_threshold: u8,
    /// Sender for UIMessage sent to the UI.
    ui_sender: Sender<UIMessage>,
    /// RNG for probability and randomly choosing a soundfile from many.
    rng: ThreadRng,
}

impl SoundManager {
    /// Create a new manager.
    /// A new manager is created every time the user reloads a soundpack.
    #[allow(clippy::cognitive_complexity)]
	pub fn new(sound_dir: &Path, ui_sender: Sender<UIMessage>) -> Result<Self> {
        let total_volume = VolumeLock::new();
        let total_is_paused = IsPausedLock::new();
        let mut sounds = Vec::new();
        let mut channel_settings = None;
        let device = default_output_device()
            .ok_or("Failed to get default audio output device.")?;
		let mut channels : BTreeMap<Box<str>, SoundChannel> = BTreeMap::new();
		channels.insert(
			String::from("misc").into_boxed_str(),
			SoundChannel::new(
                &device,
                "misc",
                total_volume.clone(),
                total_is_paused.clone()
            )
		);

        /// Traverse the soundpack in DFS. Parses XML files.
		fn visit_dir(dir: &Path, func: &mut dyn FnMut(&Path)->Result<()>) -> Result<()> {
            trace!("Directory: {:?}", dir);
            match fs::read_dir(dir) {
                Ok(entries) => for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, func)?;
                    } else if path.is_file() && path.extension().map_or(false, |ext| ext=="xml") {
                        func(&path)?;
                    }
                }
                Err(e) => {
                    warn!("Error while visiting {}: {}", dir.display(), e);
                    warn!("Will ignore this directory.");
                }
            }
            Ok(())
		}

        // Parse an XML file.
        let mut func = |file_path: &Path| -> Result<()> {
            use quick_xml::{Reader, events::Event};
            trace!(" XML: {:?}", file_path);
            let mut reader = Reader::from_file(file_path)?;
            let mut current_sound : Option<SoundEntry> = None;
            let buf = &mut Vec::new();
            loop {
                match reader.read_event(buf) {
                    // <...> or <.../>
                    Ok(Event::Start(ref data)) | Ok(Event::Empty(ref data)) => {
                        let local_name = data.local_name();
                        // <sound> or <sound/>
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
                            let mut playback_threshold: u8 = 4;
                            let files = Vec::new();
                            let weights = Vec::new();

                            for attr in data.attributes().with_checks(false) {
                                let attr = attr?;
                                // This value came from an XML file, so it must be utf8.
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
                                            channels.insert(
                                                channel_name.clone(),
                                                SoundChannel::new(
                                                    &device,
                                                    &channel_name,
                                                    total_volume.clone(),
                                                    total_is_paused.clone(),
                                                )
                                            );
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
                                        warn!(
                                            "Unknown sound value: {}",
                                            unsafe {std::str::from_utf8_unchecked(attr.key)}
                                        );
                                        warn!("Will ignore this value.");
                                    }
                                }
                            }

                            trace!("  SoundEntry");
                            if let Some(pattern) = pattern {
                                trace!("  -Pattern: {}", pattern);
                                current_sound = Some(
                                    SoundEntry{
                                        pattern,
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
                            else {
                                warn!("A SoundEntry in {:?} doesn't have a pattern!", file_path);
                                warn!("Will ignore this SoundEntry.");
                            }
                        }
                        
                        // <soundFile> or <soundFile/>
                        else if local_name == b"soundFile" {
                            if current_sound.is_none() {
                                warn!("A SoundFile in {:?} was declared outside of a valid Sound!", file_path);
                                warn!("Will ignore this SoundFile.");
                            }
                            let mut path = PathBuf::from(file_path);
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
                                    b"fileName" => path.set_file_name(attr_value),
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
                                        warn!(
                                            "Unknown sound value: {}",
                                            unsafe {std::str::from_utf8_unchecked(attr.key)}
                                        );
                                        warn!("Will ignore this value.");
                                    }
                                }
                            }
                            trace!("  --SoundFile: {:?}", path);
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

                        else if local_name == b"channelSettings" {
                            trace!("  ChannelSettings");
                            channel_settings = Some(
                                HashMap::new()
                            );
                        }
                        
                        // <channelSetting/>
                        else if local_name == b"channelSetting" {
                            if channel_settings.is_none() {
                                warn!("A ChannelSetting in {:?} was declared outside of ChannelSettings!", file_path);
                                warn!("Will ignore this ChannelSetting.");
                                continue;
                            }
                            trace!("  -ChannelSetting");
                            let mut name: Option<Box<str>> = None;
                            let mut play_type = ChannelPlayType::All;
                            for attr in data.attributes() {
                                let attr = attr?;
                                let attr_value = unsafe {std::str::from_utf8_unchecked(&attr.value)};
                                match attr.key {
                                    b"name" => {
                                        trace!("  --name: {}", attr_value);
                                        name.replace(Box::from(attr_value));
                                    }
                                    b"playType" => {
                                        trace!("  --play_type: {}", attr_value);
                                        match attr_value {
                                            "singleEager" => play_type = ChannelPlayType::SingleEager,
                                            "singleLazy" => play_type = ChannelPlayType::SingleLazy,
                                            "all" => (),
                                            other => {
                                                warn!("Unknown Channel PlayType: {}", other);
                                                warn!("Will ignore this value.");
                                            },
                                        }
                                    }
                                    _ => ()
                                }
                            }
                            if let Some(name) = name {
                                let channel_setting = ChannelSetting {
                                    play_type,
                                };
                                channel_settings.as_mut()
                                    .unwrap()
                                    .insert(name, channel_setting);
                            }
                            else {
                                warn!("A ChannelSetting is {:?} didn't specify a channel name.", file_path);
                                warn!("Will ignore this ChannelSetting.");
                            }
                        }
                    },

                    // </Sound>
                    Ok(Event::End(data)) => {
                        if current_sound.is_some() && data.local_name() == b"sound" {
                            sounds.push( current_sound.take()
                                .ok_or("Tried to finish a Sound, even though there is no Sound!")?
                            );
                        }
                    },

                    Ok(Event::Eof) => return Ok(()),

                    Err(e) => {
                        error!("Error parsing xml at position {}: {:?}", reader.buffer_position(), e);
                        return Err(
                            format!("Error parsing xml at position {}: {:?}", reader.buffer_position(), e).into()
                        )
                    },

                    _ => () // Other Reader::Events aren't used, just ignore them.
                }
            }
        };

        visit_dir(sound_dir, &mut func)?; // Run the DFS!

        // Add the default channels "total" and "music"
        let mut channel_names: Vec<Box<str>> = vec![
            "total".into(),
            "music".into(),
        ];
        for channel_name in channels.keys() {
            if !channel_names.contains(channel_name) && channel_name.as_ref() != "misc" {
                channel_names.push(channel_name.clone());
            }
        }
        // Add the "misc" channel last, so it comes last in the UI.
        channel_names.push("misc".into());
        ui_sender.send(UIMessage::LoadedSoundpack(channel_names))?;

        info!("Soundpack loaded!");
        let mut manager = Self {
            sounds,
            recent: HashSet::new(),
            previous_log: String::new(),
            ignore_list: Vec::new(),
            device,
            channels,
            total_volume,
            total_is_paused,
            total_threshold: 4,
            ui_sender,
            rng: thread_rng(),
        };

        let mut conf_path = dirs::config_dir().ok_or("No configuration directory found!")?;
        conf_path.push("soundsense-rs/default-volumes.ini");
        if conf_path.is_file() { // Check if there are default volumes.
            let file = fs::File::open(conf_path)?;
            // Apply default volumes.
            manager.get_default_volume(file)?;
        }
        // Apply channels settings if it exists.
        if let Some(channel_settings) = channel_settings {
            manager.apply_channel_settings(channel_settings);
        }

        Ok(manager)
    }

    /// Apply ChannelSettings.
    fn apply_channel_settings(&mut self, channel_settings: HashMap<Box<str>, ChannelSetting>) {
        for (name, setting) in channel_settings.iter() {
            if let Some(channel) = self.channels.get_mut(name) {
                channel.play_type = setting.play_type;
            }
        }
    }

    /// Tick down timers on recently called SoundEntries. Maintain the channels.
	pub fn maintain(&mut self, dt: usize) -> Result<()> {
		{
			let sounds = &mut self.sounds;
            let recent = &mut self.recent;
            // Tick down timeout and recent_call.
            // If the timeout == 0, remove from recent list.
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

    /// Set the volume of all, or specific channels.
    pub fn set_volume(&mut self, channel_name: &str, volume: f32) -> Result<()> {
        if channel_name == "total" {
            self.total_volume.set(volume);
        }
        else if let Some(channel) = self.channels.get_mut(channel_name) {
            channel.set_local_volume(volume);
        }
        Ok(())
    }

    pub fn set_threshold(&mut self, channel_name: &str, threshold: u8) -> Result<()> {
        if channel_name == "total" {
            self.total_threshold = threshold;
        }
        else if let Some(channel) = self.channels.get_mut(channel_name) {
            channel.set_threshold(threshold);
        }
        Ok(())
    }

    pub fn skip(&mut self, channel_name: &str) -> Result<()> {
        if channel_name == "total" {
            for (_, channel) in self.channels.iter_mut() {
                channel.skip();
            }
        }
        else if let Some(channel) = self.channels.get_mut(channel_name) {
            channel.skip();
        }
        Ok(())
    }

    pub fn play_pause(&mut self, channel_name: &str) -> Result<()> {
        if channel_name == "total" {
            let is_paused = !self.total_is_paused.flip();
            self.ui_sender.send(
                UIMessage::ChannelWasPlayPaused(
                    Box::from(channel_name),
                    is_paused
                )
            )?;
        }
        else if let Some(channel) = self.channels.get_mut(channel_name) {
            let is_paused = !channel.play_pause();
            self.ui_sender.send(
                UIMessage::ChannelWasPlayPaused(
                    Box::from(channel_name),
                    is_paused
                )
            )?;
        }
        Ok(())
    }

    pub fn finish(mut self) {
        for (_,channel) in self.channels.iter_mut() {
            channel.finish();
        }
    }

    /// Reload the ignore list.
    pub fn set_ignore_list(&mut self, ignore_list: Vec<Regex>) -> Result<()> {
        self.ignore_list = ignore_list;
        self.ui_sender.send(UIMessage::LoadedIgnoreList)?;
        Ok(())
    }

    /// Process one line of log message, and make channels play/pause/stop sounds appropriately.
    #[allow(clippy::cognitive_complexity)]
    pub fn process_log(&mut self, log: &str) -> Result<()> {
        trace!("log: {}", log);
        let mut log = log;
        lazy_static!{
            static ref REPEAT_PATTERN: Regex = Regex::new(
                r"^x[0-9]+$"
            ).unwrap();
        }
        if REPEAT_PATTERN.is_match(&log) {
            log = self.previous_log.as_str();
            trace!(" swapped: {}", log);
        }
        else {
            self.previous_log = log.to_string();
        }

        for pattern in self.ignore_list.iter() {
            if pattern.is_match(log) {
                return Ok(())
            }
        }

        let rng = &mut self.rng;
        let sounds = &mut self.sounds;
        let recent = &mut self.recent;

        for (i, sound) in sounds.iter_mut().enumerate() {
            // Activate the Sound if the log matches its pattern.
            if sound.pattern.is_match(log) {
                trace!(" pattern: {}", sound.pattern.as_str());
                recent.insert(i);
                sound.recent_call += 1;

                let mut can_play = sound.current_timeout == 0;
                if can_play {
                    if let Some(probability) = sound.probability {
                        can_play &= probability > rng.gen_range(0usize, 100usize);
                        if !can_play {
                            trace!("  can't play: failed probability roll");
                        }
                    }
                    can_play &= self.total_threshold >= sound.playback_threshold;
                    if !can_play {
                        trace!(
                            "  can't play: at threshold limit - sound.playback_threshold: {}, total_threshold: {}",
                            sound.playback_threshold, self.total_threshold
                        );
                    }
                } else {
                    trace!("  can't play: current_timeout: {}", sound.current_timeout);
                }

                if can_play {
                    let files = &sound.files;
                    // Choose index.
                    // If there are more than one soundfiles,
                    //      and the sound doesn't loop, choose based on weighted distribution.
                    // Else, 0.
                    let idx : usize = if files.len() > 1 && !sound.loop_attr.unwrap_or(false) {
                        match WeightedIndex::new(&sound.weights) {
                            Ok(weight) => weight.sample(rng),
                            Err(e) => {
                                trace!("Error while weighing files: {}", e);
                                0
                            }
                        }
                    } else {
                        0
                    };

                    // Play on a given channel.
                    if let Some(chn) = &sound.channel {
                        trace!("  channel: {}", chn);
                        let channel = if let Some(channel) = self.channels.get_mut(chn) {
                            channel
                        } else {
                            trace!("   doesn't exist in current soundpack!");
                            continue;
                        };
                        let chn_len = channel.len();
                        let chn_threshold = channel.get_threshold();
                        // Check if there are too many sounds playing on this channel (concurrency).
                        if chn_len >= sound.concurency.unwrap_or(std::usize::MAX) {
                            trace!("   can't play: at concurency limit: limit {}, channel {}",
                                sound.concurency.unwrap(), chn_len);
                        }
                        // Check if the playback_threshold is higher than the channel threshold.
                        else if chn_threshold < sound.playback_threshold {
                            trace!("   can't play: at threshold limit - sound.playback_threshold: {}, channel_threshold: {}",
                                sound.playback_threshold, chn_threshold);
                        }
                        else {
                            // Set current_timeout if the sound has a timeout value.
                            if let Some(timeout) = sound.timeout {
                                sound.current_timeout = timeout;
                            }
                            let device = &self.device;
                            
                            // Check if the sound starts a loop
                            if let Some(is_loop_start) = sound.loop_attr {
                                if is_loop_start {
                                    trace!("   loop=start");
                                    channel.change_loop(device, sound.files.as_slice(), sound.delay.unwrap_or(0), rng);
                                } else {
                                    // If loop=stop, add the sound to the oneshot player.
                                    trace!("   loop=stop");
                                    channel.stop_loop(sound.delay.unwrap_or(0));
                                    if !sound.files.is_empty() {
                                        channel.add_oneshot(device, &files[idx], sound.delay.unwrap_or(0), rng);
                                    }
                                }
                            }
                            // Otherwise, add to oneshot player.
                            else if !sound.files.is_empty() && channel.len() <= sound.concurency.unwrap_or(std::usize::MAX) {
                                channel.add_oneshot(device, &files[idx], sound.delay.unwrap_or(0), rng);
                            }
                        }
                    }
                    else if !sound.files.is_empty() {
                        trace!("  channel: misc");
                        let channel = self.channels.get_mut("misc").unwrap();
                        let chn_len = channel.len();
                        let chn_threshold = channel.get_threshold();
                        if chn_len >= sound.concurency.unwrap_or(std::usize::MAX) {
                            trace!("   can't play: at concurency limit - limit {}, channel {}",
                                sound.concurency.unwrap(), chn_len);
                        }
                        else if chn_threshold < sound.playback_threshold {
                            trace!("   can't play: at threshold limit - sound.playback_threshold: {}, channel_threshold: {}",
                                sound.playback_threshold, chn_threshold);
                        }
                        else {
                            if let Some(timeout) = sound.timeout {
                                sound.current_timeout = timeout;
                            }
                            channel.add_oneshot(&self.device, &files[idx], sound.delay.unwrap_or(0), rng);
                        }
                    }
                }

                if sound.halt_on_match {
                    return Ok(())
                }
            }
        }
        Ok(())
    }

    /// Write the current slider values into the soundsense-rs/default-volumes.ini file.
    pub fn set_current_volumes_as_default(&self, mut file: File) -> Result<()> {
        use std::io::Write;
        writeln!(&mut file, "all={}", (self.total_volume.get()*100.0) as u32)?;
        for (channel_name, channel) in self.channels.iter() {
            writeln!(&mut file, "{}={}", channel_name, (channel.get_local_volume()*100.0) as u32)?;
        }
        Ok(())
    }

    /// Get the volume from the soundsense-rs/default-volumes.ini file.
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
                if name == "total" {
                    self.total_volume.set(volume / 100.0);
                }
                else if let Some(chn) = self.channels.get_mut(name) {
                    chn.set_local_volume(volume / 100.0);
                }
                entries.push((name.to_string().into_boxed_str(), volume));
            }
        }
        // Tell the UI to change the slider values.
        self.ui_sender
            .send(UIMessage::LoadedVolumeSettings(entries))?;
        Ok(())
    }
}

/// Convert a playlist into a list or file paths.
fn parse_playlist(path: &Path) -> Result<Vec<PathBuf>> {
    let parent_path = path.parent().unwrap();

    let mut path_vec = Vec::new();
    let f = File::open(path)?;
    let f = BufReader::new(f);
    // Check if the path contains the m3u or pls extension.
    // Else, error out.
    let extension = path.extension()
        .filter(|ext| *ext=="m3u" || *ext=="pls")
        .ok_or_else(|| format!(
            "Playlist {:?} is not valid! Playlist needs to have either .m3u or .pls extension.",
            path
        ))?;
    if extension == "m3u" {
        for line in f.lines()
            .filter_map(|l| l.ok())
        {
            lazy_static! {
                static ref M3U_PATTERN: Regex = Regex::new(
                    r"#EXT.*"
                ).unwrap();
            }

            if !M3U_PATTERN.is_match(&line) {
                let mut path = PathBuf::from(parent_path);
                path.push(line);
                trace!("   Playlist Entry: {:?}", path);
                path_vec.push(path);
            }
        }
    }
    else if extension == "pls" {
        for line in f.lines()
            .filter_map(|l| l.ok())
        {
            lazy_static! {
                static ref PLS_PATTERN: Regex = Regex::new(
                    r"File.+=(.+)"
                ).unwrap();
            }
            
            if let Some(caps) = PLS_PATTERN.captures(&line) {
                let mut path = PathBuf::from(parent_path);
                path.push(&caps[0]);
                trace!("   Playlist Entry: {:?}", path);
                path_vec.push(path);
            }
        }
    }
    Ok(path_vec)
}