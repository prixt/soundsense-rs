use std::sync::mpsc::Sender;
use web_view::*;
use crate::message::{SoundMessage, VolumeChange};

static HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));

pub fn run(
    tx: Sender<SoundMessage>,
    gamelog_path: Option<std::path::PathBuf>,
    soundpack_path: Option<std::path::PathBuf>,
    ignore_path: Option<std::path::PathBuf>,
) {
    let webview = builder()
        .title("SoundSense-rs")
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
                    tx.send(SoundMessage::ChangeGamelog(path)).unwrap()
                }
                "load_soundpack" => if let Some(path) = webview.dialog()
                    .choose_directory("Choose soundpack directory", "")
                    .unwrap() {
                    tx.send(SoundMessage::ChangeSoundpack(path, UIHandle::new(webview.handle()))).unwrap()
                }
                "load_ignore_list" => if let Some(path) = webview.dialog()
                    .open_file("Choose ignore.txt", "")
                    .unwrap() {
                    tx.send(SoundMessage::ChangeIgnoreList(path)).unwrap()
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
                other => {
                    if let Ok(VolumeChange{channel, volume}) = serde_json::from_str(other) {
                        tx.send(SoundMessage::VolumeChange(channel, volume)).unwrap()
                    } else {
                        unreachable!("Unrecognized argument: {}", other)
                    }
                }
            }
            Ok(())
        })
        .build()
        .unwrap();
    
    if let Some(path) = gamelog_path {
        tx.send(SoundMessage::ChangeGamelog(path)).unwrap();
    }
    if let Some(path) = soundpack_path {
        tx.send(SoundMessage::ChangeSoundpack(path, UIHandle::new(webview.handle()))).unwrap();
    }
    if let Some(path) = ignore_path {
        tx.send(SoundMessage::ChangeIgnoreList(path)).unwrap();
    }
    
    webview.run().unwrap();
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
    pub fn add_slider(&mut self, name: String) {
        let name = name.into_boxed_str();
        if !self.channels.contains(&name){
            self.channels.push(name.clone());
            self.handle.dispatch(
                move |webview| {
                    let script = format!(
r#"
let channels = document.getElementById('channels');
channels.insertAdjacentHTML(
    'beforeend',
    "<tr class='w3-row'> \
        <td class='w3-center' style='width:50px'><h4>{channel_name}</h4></td> \
        <td class='w3-rest'> \
            <input type='range' \
                name='{channel_name}_slider' \
                id='{channel_name}_slider' \
                min='0' \
                max='100' \
                value='100' \
            /> \
        </td> \
    </tr>"
);

let slider = document.getElementById("{channel_name}_slider");
slider.addEventListener(
    /MSIE|Trident|Edge/.test(window.navigator.userAgent) ? 'change' : 'input',
    function() {{
        external.invoke('{{"channel":"{channel_name}", "volume":'+this.value+'}}');
    }},
    false
);
"#,
                    channel_name=&name);
                    webview.eval(&script)
                }
            ).unwrap();
        }
    }
    pub fn clear_sliders(&mut self) {
        self.handle.dispatch(
            |webview| {
                webview.eval(
r#"
let channels = document.getElementById("channels");
while (channels.firstChild) {
    channels.removeChild(channels.firstChild);
}
"#
                )
            }
        ).unwrap();
    }
}