use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};

use notify::DebouncedEvent;

mod sound_manager;
use sound_manager::SoundManager;

pub fn sound_thread(
	log_path: &Path,
	sound_path: &Path,
	log_receiver: Receiver<DebouncedEvent>,
	mut receiver: Receiver<String>,
	mut sender: Sender<(String, String)>,
) {
	use std::thread::sleep;
	use std::time::Duration;
	const SLEEP_DURATION: Duration = Duration::from_micros(100);

	let mut file = fs::File::open(log_path).unwrap();
    let buffer = &mut Vec::new();
	file.seek(SeekFrom::End(0)).unwrap();
	let mut sound_manager = SoundManager::new(sound_path);

	loop {
		for e in log_receiver.try_iter() {
			if let DebouncedEvent::Write(_) = e {
				file.read_to_end(buffer).unwrap();
				let messages = String::from_utf8_lossy(&buffer);
				for line in messages.lines() {
					sound_manager.process_log(line.trim());
				}
				buffer.clear()
			}
		}
		sound_manager.maintain(&mut sender);
		sleep(SLEEP_DURATION);
	}
}