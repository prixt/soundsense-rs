#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;

use std::env;
use std::sync::mpsc::channel;
use std::path::PathBuf;

mod sound;
mod ui;
mod message;

fn main() {
	let args: Vec<String> = env::args().collect();
	let mut opts = getopts::Options::new();
	opts.optopt("l", "gamelog", "Path to the gamelog.txt file.", "LOG_FILE");
	opts.optopt("p", "soundpack", "Path to the soundpack directory.", "PACK_DIR");
	opts.optopt("i", "ignore", "Path to the ignore.txt file.", "IGNORE_FILE");

	let mut gamelog_path: Option<PathBuf> = None;
	let mut soundpack_path: Option<PathBuf> = None;
	let mut ignore_path: Option<PathBuf> = None;

	let matches = opts.parse(&args[1..]).unwrap();
	if let Some(path) = matches.opt_str("l") {
		let path = PathBuf::from(path);
		if path.is_file() {
			gamelog_path = Some(path);
		}
	} else {
		let path = PathBuf::from("./gamelog.txt");
		if path.is_file() {
			gamelog_path = Some(path);
		}
	}
	if let Some(path) = matches.opt_str("p") {
		let path = PathBuf::from(path);
		if path.is_dir() {
			soundpack_path = Some(path);
		}
	} else {
		let path = PathBuf::from("./soundpack");
		if path.is_dir() {
			soundpack_path = Some(path);
		}
	}
	if let Some(path) = matches.opt_str("i") {
		let path = PathBuf::from(path);
		if path.is_file() {
			ignore_path = Some(path);
		}
	} else {
		let path = PathBuf::from("./ignore.txt");
		if path.is_file() {
			ignore_path = Some(path);
		}
	}

	let (tx, rx) = channel();
	std::thread::Builder::new()
		.name("sound_thread".to_string())
		.spawn(move || sound::run(rx)).unwrap();
	ui::run(tx, gamelog_path, soundpack_path, ignore_path);
}