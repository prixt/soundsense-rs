use std::sync::{Mutex, mpsc::{Sender, Receiver}};
use web_view::*;
use crate::message::{SoundMessage, UIMessage};
// use crate::download;

pub fn run(
    sound_tx: Sender<SoundMessage>, ui_rx: Receiver<UIMessage>,
    gamelog_path: Option<std::path::PathBuf>,
    soundpack_path: Option<std::path::PathBuf>,
    ignore_path: Option<std::path::PathBuf>,
) {
    static HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));
    
    if let Some(path) = &soundpack_path {
        sound_tx.send(SoundMessage::ChangeSoundpack(path.clone())).unwrap();
    }
    if let Some(path) = &gamelog_path {
        sound_tx.send(SoundMessage::ChangeGamelog(path.clone())).unwrap();
    }
    if let Some(path) = &ignore_path {
        sound_tx.send(SoundMessage::ChangeIgnoreList(path.clone())).unwrap();
    }
    
    let gamelog_path = Mutex::new(gamelog_path);
    let soundpack_path = Mutex::new(soundpack_path);
    let ignore_path = Mutex::new(ignore_path);
    
    let mut webview = builder()
        .title("SoundSense-RS")
        .content(Content::Html(HTML))
        .size(500, 550)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            match arg {
                "load_gamelog" => if let Some(path) = webview.dialog()
                    .open_file("Choose gamelog.txt", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeGamelog(path.clone())).unwrap();
                    gamelog_path.lock()
                        .unwrap()
                        .replace(path);
                }
                "load_soundpack" => if let Some(path) = webview.dialog()
                    .choose_directory("Choose soundpack directory", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeSoundpack(path.clone())).unwrap();
                    soundpack_path.lock()
                        .unwrap()
                        .replace(path);
                }
                "load_ignore_list" => if let Some(path) = webview.dialog()
                    .open_file("Choose ignore.txt", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeIgnoreList(path.clone())).unwrap();
                    ignore_path.lock()
                        .unwrap()
                        .replace(path);
                }
                "show_about" => {
                    webview.dialog()
                        .info(
                            "SoundSense-rs",
                            r"Created by prixt
The original SoundSense can be found at:
    http://df.zweistein.cz/soundsense/
Source at:
    https://github.com/prixt/soundsense-rs",
                        ).unwrap()
                }
                // }
                // "set_current_paths_as_default" => {
                //     sound_tx.send(SoundMessage::SetCurrentPathsAsDefault).unwrap()
                // }
                // "set_current_volumes_as_default" => {
                //     sound_tx.send(SoundMessage::SetCurrentVolumesAsDefault).unwrap()
                // }
                "link_original" => {
                    if let Err(e) = webbrowser::open("http://df.zweistein.cz/soundsense/") {
                        eprintln!("webbrowser error: {}", e);
                    }
                }
                "link_fork" => {
                    if let Err(e) = webbrowser::open("https://github.com/jecowa/soundsensepack") {
                        eprintln!("webbrowser error: {}", e);
                    }
                }
                other => {
                    let parts: Vec<&str> = other.split(':').collect();
                    if parts[0] == "change_volume" {
                        let channel_name: Box<str> = parts[1].into();
                        let channel_volume: f32 = parts[2].parse().unwrap();
                        sound_tx.send(
                            SoundMessage::VolumeChange(channel_name, channel_volume)
                        ).unwrap();
                    }
                    else {
                        unimplemented!("Unimplemented webview argument: {}", other);
                    }
                }
            }
            Ok(())
        })
        .build()
        .unwrap();
    let mut handle = UIHandle::new(webview.handle());
    
    while let Some(result) = webview.step() {
        result.unwrap();
        for ui_message in ui_rx.try_iter() {

            #[allow(clippy::single_match)]
            match ui_message {
                UIMessage::LoadedSoundpack(channel_names) => {
                    handle.clear_sliders();
                    for name in channel_names.iter() {
                        handle.add_slider(name)
                    }
                }
                _ => ()
            }
        }
    }

    use std::io::Write;
    let mut conf_path = dirs::config_dir().unwrap();
    conf_path.push("soundsense-rs");
    if !conf_path.is_dir() {
        std::fs::create_dir(&conf_path)
            .expect("Failed to create soundsense-rs config directory.");
    }
    conf_path.push("conf.ini");
    let mut conf_file = std::fs::File::create(conf_path)
        .expect("Failed to create conf.ini file.");
    if let Some(path) = gamelog_path.lock().unwrap().as_ref() {
        writeln!(conf_file, "gamelog={}", path.to_string_lossy()).unwrap();
    };
    if let Some(path) = soundpack_path.lock().unwrap().as_ref() {
        writeln!(conf_file, "soundpack={}", path.to_string_lossy()).unwrap();
    };
    if let Some(path) = ignore_path.lock().unwrap().as_ref() {
        writeln!(conf_file, "ignore={}", path.to_string_lossy()).unwrap();
    };
}

pub struct UIHandle {
    handle: Handle<()>,
    channels: Vec<Box<str>>,
}

impl UIHandle {
    pub fn new(handle: Handle<()>) -> Self {
        Self {
            handle, channels: vec![],
        }
    }
    pub fn add_slider(&mut self, name: &str) {
        let name = name.into();
        if !self.channels.contains(&name){
            self.channels.push(name.clone());
            self.handle.dispatch(
                move |webview| {
                    webview.eval(
                        &format!(r#"addSlider("{channel_name}")"#, channel_name=&name)
                    )
                }
            ).unwrap();
        }
    }
    pub fn clear_sliders(&mut self) {
        self.channels.clear();
        self.handle.dispatch(
            |webview| {
                webview.eval("clearSliders()")
            }
        ).unwrap();
    }
}