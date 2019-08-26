#[macro_use]
extern crate serde_derive;

use std::sync::mpsc::channel;

mod sound;
mod ui;
mod message;

fn main() {
	let (tx, rx) = channel();
	std::thread::Builder::new()
		.name("sound_thread".to_string())
		.spawn(move || sound::run(rx)).unwrap();
	ui::run(tx, None, None);
}