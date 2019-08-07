use std::env;
use std::sync::mpsc::channel;
use gio::prelude::*;

mod sound;
mod ui;
mod message;
use message::SoundMessage;

fn main() {
    let app = gtk::Application::new(None, gio::ApplicationFlags::FLAGS_NONE)
        .expect("initialization failed!");
    
    let (sound_tx, sound_rx) = channel::<SoundMessage>();

    std::thread::Builder::new()
        .name("sound_thread".to_string())
        .spawn(move || sound::run(sound_rx)).unwrap();

    app.connect_activate(move |app|{
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        sound_tx.send(message::SoundMessage::GlibSender(tx)).unwrap();
        ui::build_ui(app, sound_tx.clone(), rx);
    });

    app.run(&env::args().collect::<Vec<_>>());
}