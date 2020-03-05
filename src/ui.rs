use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::sync::Mutex;
use crossbeam::channel::{Sender, Receiver};
use web_view::*;
use crate::message::{SoundMessage, UIMessage};

/// The UI thread function.
pub fn run(
    sound_tx: Sender<SoundMessage>, ui_rx: Receiver<UIMessage>,
    gamelog_path: Option<PathBuf>,
    soundpack_path: Option<PathBuf>,
    ignore_path: Option<PathBuf>,
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
        .size(500, 440)
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
                    remove_alert(webview, "gamelog_loaded");
                    add_alert(webview, "loading_gamelog", "blue", "&#x231B; Loading gamelog...");
                }
                "load_soundpack" => if let Some(path) = webview.dialog()
                    .choose_directory("Choose soundpack directory", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeSoundpack(path.clone())).unwrap();
                    soundpack_path.lock()
                        .unwrap()
                        .replace(path);
                    remove_alert(webview, "soundpack_loaded");
                    add_alert(webview, "loading_soundpack", "blue", "&#x231B; Loading soundpack...");
                }
                "load_ignore_list" => if let Some(path) = webview.dialog()
                    .open_file("Choose ignore.txt", "")
                    .unwrap() {
                    sound_tx.send(SoundMessage::ChangeIgnoreList(path.clone())).unwrap();
                    ignore_path.lock()
                        .unwrap()
                        .replace(path);
                    remove_alert(webview, "ignore_loaded");
                    add_alert(webview, "loading_ignore", "blue", "&#x231B; Loading ignore list...");
                }
                "show_about" => {
                    webview.dialog()
                        .info(
                            "About SoundSense-RS",
                            &format!(
r"A sound-engine utility for Dwarf Fortress, written in Rust
    Version {}
    Created by prixt
    Original SoundSense created by zwei:
        http://df.zweistein.cz/soundsense/",
                                env!("CARGO_PKG_VERSION")
                            )
                        ).unwrap();
                }
                "link_original" => {
                    if let Err(e) = webbrowser::open("http://df.zweistein.cz/soundsense/") {
                        warn!("Failed to open link with system's default browser: {}", e);
                        add_error(webview, "Webbrowser Error",
                            "Failed to open link with the system's default browser.");
                    }
                }
                "link_fork" => {
                    if let Err(e) = webbrowser::open("https://github.com/jecowa/soundsensepack") {
                        warn!("Failed to open link with system's default browser: {}", e);
                        add_error(webview, "Webbrowser Error",
                            "Failed to open link with the system's default browser.");
                    }
                }
                "link_source" => {
                    if let Err(e) = webbrowser::open("https://github.com/prixt/soundsense-rs") {
                        warn!("Failed to open link with system's default browser: {}", e);
                        add_error(webview, "Webbrowser Error",
                            "Failed to open link with the system's default browser.");
                    }
                }
                "set_default_paths" => {
                    let mut conf_path = dirs::config_dir()
                        .expect("Failed to get configuration directory.");
                    conf_path.push("soundsense-rs");
                    if !conf_path.is_dir() {
                        std::fs::create_dir(&conf_path)
                            .expect("Failed to create soundsense-rs config directory.");
                    }
                    conf_path.push("default-paths.ini");
                    let mut conf_file = fs::File::create(conf_path)
                        .expect("Failed to create default-paths.ini file.");
                    if let Some(path) = gamelog_path.lock().unwrap().as_ref() {
                        writeln!(conf_file, "gamelog={}", path.to_string_lossy()).unwrap();
                    };
                    if let Some(path) = soundpack_path.lock().unwrap().as_ref() {
                        writeln!(conf_file, "soundpack={}", path.to_string_lossy()).unwrap();
                    };
                    if let Some(path) = ignore_path.lock().unwrap().as_ref() {
                        writeln!(conf_file, "ignore={}", path.to_string_lossy()).unwrap();
                    };
                    remove_alert(webview, "remove_default_paths");
                    add_alert(webview, "set_default_paths", "green", "&#x1F4BE; Default paths set.");
                }
                "set_default_volumes" => {
                    let mut conf_path = dirs::config_dir()
                        .expect("Failed to get configuration directory.");
                    conf_path.push("soundsense-rs");
                    if !conf_path.is_dir() {
                        fs::create_dir(&conf_path)
                            .expect("Failed to create soundsense-rs config directory.");
                    }
                    conf_path.push("default-volumes.ini");
                    let conf_file = fs::File::create(conf_path)
                        .expect("Failed to create default-volumes.ini file.");
                    sound_tx.send(SoundMessage::SetCurrentVolumesAsDefault(conf_file)).unwrap();
                    remove_alert(webview, "remove_default_volumes");
                    add_alert(webview, "set_default_volumes", "green", "&#x1F4BE; Default volumes set.");
                }
                "remove_default_paths" => {
                    let mut conf_path = dirs::config_dir()
                        .expect("Failed to get configuration directory.");
                    conf_path.push("soundsense-rs");
                    if conf_path.is_dir() {
                        conf_path.push("default-paths.ini");
                        if conf_path.is_file() {
                            fs::remove_file(conf_path)
                                .expect("Failed to delete default-paths.ini file.");
                            remove_alert(webview, "set_default_paths");
                            add_alert(webview, "remove_default_paths", "blue", "&#x1F5D1; Removed path defaults.");
                        }
                    }
                }
                "remove_default_volumes" => {
                    let mut conf_path = dirs::config_dir().unwrap();
                    conf_path.push("soundsense-rs");
                    if conf_path.is_dir() {
                        conf_path.push("default-volumes.ini");
                        if conf_path.is_file() {
                            fs::remove_file(conf_path)
                                .expect("Failed to delete default-volumes.ini file.");
                            remove_alert(webview, "set_default_volumes");
                            add_alert(webview, "remove_default_volumes", "blue", "&#x1F5D1; Removed volume defaults.");
                        }
                    }
                }
                other => {
                    let parts: Vec<&str> = other.split(':').collect();
                    match parts[0] {
                        "change_volume" => {
                            let channel_name: Box<str> = parts[1].into();
                            let channel_volume: f32 = parts[2].parse().unwrap();
                            sound_tx.send(
                                SoundMessage::VolumeChange(channel_name, channel_volume)
                            ).unwrap();
                        }
                        "change_threshold" => {
                            let channel_name: Box<str> = parts[1].into();
                            let channel_threshold: u8 = parts[2].parse().unwrap();
                            sound_tx.send(
                                SoundMessage::ThresholdChange(channel_name, channel_threshold)
                            ).unwrap();
                        }
                        "skip_current_sound" => {
                            let channel_name: Box<str> = parts[1].into();
                            sound_tx.send(
                                SoundMessage::SkipCurrentSound(channel_name)
                            ).unwrap();
                        }
                        "play_pause" => {
                            let channel_name: Box<str> = parts[1].into();
                            sound_tx.send(
                                SoundMessage::PlayPause(channel_name)
                            ).unwrap();
                        }
                        "test_message" => {
                            info!("UI test message: {}", parts[1]);
                        }
                        _ => {
                            add_error(
                                webview,
                                "Webview Error",
                                &format!("Unimplemented webview argument: {}", other)
                            );
                        }
                    }
                }
            }
            Ok(())
        })
        .build()
        .unwrap();
    
    webview.step().unwrap().unwrap();
    
    // Keep activating the webview event loop.
    while let Some(result) = webview.step() {
        result.unwrap();
        for ui_message in ui_rx.try_iter() {
            match ui_message {
                UIMessage::LoadedSoundpack(channel_names) => {
                    clear_sliders(&mut webview);
                    for name in channel_names.iter() {
                        add_slider(&mut webview, name);
                    }
                    remove_alert(&mut webview, "loading_soundpack");
                    add_alert(&mut webview, "soundpack_loaded", "green", "&#x2714; Soundpack loaded!");
                }
                UIMessage::LoadedVolumeSettings(entries) => {
                    for (name, volume) in entries.into_iter() {
                        set_slider_value(&mut webview, name, volume);
                    }
                }
                UIMessage::LoadedGamelog => {
                    remove_alert(&mut webview, "loading_gamelog");
                    add_alert(&mut webview, "gamelog_loaded", "green", "&#x2714; Gamelog loaded!");
                }
                UIMessage::LoadedIgnoreList => {
                    remove_alert(&mut webview, "loading_ignore");
                    add_alert(&mut webview, "ignore_loaded", "green", "&#x2714; Ignore list loaded!");
                }
                UIMessage::ChannelWasPlayPaused(name, is_paused) => {
                    set_slider_head(&mut webview, &name, is_paused);
                }
                UIMessage::SoundThreadPanicked(name, text) => {
                    clear_sliders(&mut webview);
                    add_error(&mut webview, &name, &text);
                }
            }
        }
    }
}

/// add a slider for a channel with the give name
fn add_slider(webview: &mut WebView<()>, name: &str) {
    webview.eval(
        &format!(r#"addSlider("{channel_name}")"#, channel_name=name)
    ).unwrap();
}
/// set the slider value for the named channel
fn set_slider_value(webview: &mut WebView<()>, name: Box<str>, value: f32) {
    webview.eval(&format!(
        r#"setSliderValue("{channel_name}", {value})"#,
        channel_name=name,
        value=value as u32
    )).unwrap();
}
/// remove all sliders
fn clear_sliders(webview: &mut WebView<()>) {
    webview.eval("clearSliders()").unwrap();
}
fn set_slider_head(webview: &mut WebView<()>, channel_name: &str, is_paused: bool) {
    if is_paused {
        webview.eval(&format!(
            r#"setSliderHead("{}", true)"#,
            channel_name,
        )).unwrap();
    }
    else {
        webview.eval(&format!(
            r#"setSliderHead("{}", false)"#,
            channel_name,
        )).unwrap();
    }
}
/// display a notice for the user.
fn add_alert(webview: &mut WebView<()>, name: &str, color: &str, text: &str) {
    webview.eval(&format!(
        r#"addAlert("{}", "{}", "{}")"#,
        name, color, text
    )).unwrap();
}
/// remove a notice if it exists.
fn remove_alert(webview: &mut WebView<()>, name: &str) {
    webview.eval(&format!(
        r#"removeAlert("{}")"#,
        name
    )).unwrap();
}
/// display an error message for the user.
fn add_error(webview: &mut WebView<()>, name: &str, text: &str) {
    webview.eval(&format!(
        r#"addError("{}", "{}")"#,
        name, text
    )).unwrap();
}