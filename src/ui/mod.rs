use std::sync::mpsc::{Sender, Receiver};
// use std::sync::atomic::{AtomicBool, Ordering};
use web_view::*;
use crate::message::{SoundMessage, UIMessage};
// use lazy_static::*;
// use crate::download;

pub fn run(
    sound_tx: Sender<SoundMessage>, ui_rx: Receiver<UIMessage>,
    gamelog_path: Option<std::path::PathBuf>,
    soundpack_path: Option<std::path::PathBuf>,
    ignore_path: Option<std::path::PathBuf>,
) {
    static HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));
    
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
                    sound_tx.send(SoundMessage::ChangeGamelog(path)).unwrap()
                }
                "load_soundpack" => if let Some(path) = webview.dialog()
                    .choose_directory("Choose soundpack directory", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeSoundpack(path)).unwrap()
                }
                "load_ignore_list" => if let Some(path) = webview.dialog()
                    .open_file("Choose ignore.txt", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeIgnoreList(path)).unwrap()
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
                // "download_soundpack" => {
                    // lazy_static! {
                    //     static ref IS_DOWNLOADING: AtomicBool = AtomicBool::new(false); 
                    // }
                    // if dbg!(!IS_DOWNLOADING.swap(true, Ordering::SeqCst)) {
                    //     let handle1 = webview.handle();
                    //     let handle2 = webview.handle();
                    //     std::thread::Builder::new()
                    //         .name("download_thread".into())
                    //         .spawn(move || download::run(&IS_DOWNLOADING, handle1, handle2))
                    //         .unwrap();
                    // } else {
                    //     webview.dialog().warning(
                    //         "Already downloading!",
                    //         "SoundSense-rs is currently already downloading the soundpack."
                    //     ).unwrap()
                    // }
                // }
                "set_current_paths_as_default" => {
                    sound_tx.send(SoundMessage::SetCurrentPathsAsDefault).unwrap()
                }
                "set_current_volumes_as_default" => {
                    sound_tx.send(SoundMessage::SetCurrentVolumesAsDefault).unwrap()
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
    
    if let Some(path) = gamelog_path {
        sound_tx.send(SoundMessage::ChangeGamelog(path)).unwrap();
    }
    if let Some(path) = soundpack_path {
        sound_tx.send(SoundMessage::ChangeSoundpack(path)).unwrap();
    }
    if let Some(path) = ignore_path {
        sound_tx.send(SoundMessage::ChangeIgnoreList(path)).unwrap();
    }
    
    while let Some(result) = webview.step() {
        result.unwrap();
        for ui_message in ui_rx.try_iter() {
            match ui_message {
                UIMessage::LoadedSoundpack(channel_names) => {
                    handle.clear_sliders();
                    for name in channel_names.iter() {
                        handle.add_slider(name)
                    }
                }
                UIMessage::LoadedGamelog => {

                }
                UIMessage::LoadedIgnoreList => {

                }
            }
        }
    }
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