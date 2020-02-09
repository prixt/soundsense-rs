use std::env;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender, Receiver};
use iced::*;
use tinyfiledialogs as tfd;
use crate::message::*;

pub fn run() {
    let mut setting = Settings::default();
    setting.window = window::Settings {
        size: (600, 400),
        resizable: true,
        decorations: true,
    };
    UIStruct::run(setting)
}

struct ChannelSlider {
    channel_name: String,
    channel_volume: f32,
    channel_slider: slider::State,
}

struct UIStruct {
    gamelog_button: button::State,
    soundpack_button: button::State,
    ignore_button: button::State,
    about_button: button::State,
    scroll_state: scrollable::State,

    volumes: Vec<ChannelSlider>,
    sound_sender: Sender<SoundMessage>,
    ui_receiver: Receiver<Vec<String>>,
}

impl Application for UIStruct {
    type Message = UIMessage;
    type Executor = executor::Default;

    fn new() -> (Self, Command<Self::Message>) {
        use UIMessage::*;

        let (sound_sender, sound_receiver) = channel();
        let (ui_sender, ui_receiver) = channel();
        std::thread::Builder::new()
            .name("sound_thread".to_string())
            .spawn(move || crate::sound::run(sound_receiver, ui_sender)).unwrap();

        let args: Vec<String> = env::args().collect();
        let mut opts = getopts::Options::new();
        opts.optopt("l", "gamelog", "Path to the gamelog.txt file.", "LOG_FILE");
        opts.optopt("p", "soundpack", "Path to the soundpack directory.", "PACK_DIR");
        opts.optopt("i", "ignore", "Path to the ignore.txt file.", "IGNORE_FILE");
    
        let matches = opts.parse(&args[1..]).unwrap();
        let mut init_commands = vec!();
        if let Some(path) = matches.opt_str("l")
            .and_then(|path| {
                let path = PathBuf::from(path);
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
            .or_else(|| {
                let path = PathBuf::from("./gamelog.txt");
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
        {
            let tx = sound_sender.clone();
            init_commands.push(
                Command::perform(
                    async move {
                        tx.send(
                            SoundMessage::ChangeGamelog(path)
                        ).unwrap();
                    },
                    |()| Resolved
                )
            )
        }

        if let Some(path) = matches.opt_str("p")
            .and_then(|path| {
                let path = PathBuf::from(path);
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            })
            .or_else(|| {
                let path = PathBuf::from("./soundpack");
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            })
        {
            let tx = sound_sender.clone();
            init_commands.push(
                Command::perform(
                    async move {
                        tx.send(
                            SoundMessage::ChangeSoundpack(path)
                        ).unwrap();
                    },
                    |()| SoundpackLoaded
                )
            )
        }

        if let Some(path) = matches.opt_str("i")
            .and_then(|path| {
                let path = PathBuf::from(path);
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
            .or_else(|| {
                let path = PathBuf::from("./ignore.txt");
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
        {
            let tx = sound_sender.clone();
            init_commands.push(
                Command::perform(
                    async move {
                        tx.send(
                            SoundMessage::ChangeIgnoreList(path)
                        ).unwrap();
                    },
                    |()| Resolved
                )
            )
        }

        let app = UIStruct {
            gamelog_button: button::State::default(),
            soundpack_button: button::State::default(),
            ignore_button: button::State::default(),
            about_button: button::State::default(),
            scroll_state: scrollable::State::default(),

            volumes: Vec::new(),
            sound_sender,
            ui_receiver
        };

        (
            app,
            Command::batch(init_commands)
        )
    }

    fn title(&self) -> String {
        String::from("SoundSense-rs")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use UIMessage::*;

        match message {
            GamelogPressed => {
                let sound_sender = self.sound_sender.clone();
                Command::perform(
                    async move {
                        if let Some(file_path) = tfd::open_file_dialog(
                            "Open gamelog.txt",
                            "gamelog.txt",
                            Some((&["*.txt"], ".txt"))
                        ) {
                            sound_sender.send(SoundMessage::ChangeGamelog(file_path.into()))
                                .expect("Failed to send SoundMessage::ChangeGamelog");
                        }
                    },
                    |()| Resolved
                )
            }

            SoundpackPressed => {
                let sound_sender = self.sound_sender.clone();
                Command::perform(
                    async move {
                        if let Some(file_path) = tfd::select_folder_dialog(
                            "Select soundpack directory",
                            "soundpack"
                        ) {
                            sound_sender.send(SoundMessage::ChangeSoundpack(file_path.into()))
                                .expect("Failed to send SoundMessage::ChangeSoundpack");
                            true
                        } else {
                            false
                        }
                    },
                    |path_found| if path_found {
                        SoundpackLoaded
                    } else {
                        Resolved
                    }
                )
            } 
            SoundpackLoaded => {
                let channels = self.ui_receiver.recv().unwrap();
                self.volumes = channels
                    .into_iter()
                    .map(|channel_name| {
                        ChannelSlider {
                            channel_name,
                            channel_volume: 100.0,
                            channel_slider: slider::State::default(),
                        }
                    })
                    .collect();
                Command::none()
            }

            IgnorePressed => {
                let sound_sender = self.sound_sender.clone();
                Command::perform(
                    async move {
                        if let Some(file_path) = tfd::open_file_dialog(
                            "Open ignore.txt",
                            "ignore.txt",
                            Some((&["*.txt"], ".txt"))
                        ) {
                            sound_sender.send(SoundMessage::ChangeIgnoreList(file_path.into()))
                                .expect("Failed to send SoundMessage::ChangeIgnoreList");
                        }
                    },
                    |()| Resolved
                )
            }

            AboutPressed => Command::perform(
                async {
                    tfd::message_box_ok(
                        "About",
                        r"Created by prixt
The original SoundSense can be found at:
    http://df.zweistein.cz/soundsense/
Source at:
    https://github.com/prixt/soundsense-rs",
                        tfd::MessageBoxIcon::Info
                    )
                },
                |()| Resolved
            ),

            ChannelVolumeChanged(idx, volume) => {
                let mut volume_slider = unsafe {self.volumes.get_unchecked_mut(idx)};
                volume_slider.channel_volume = volume;
                self.sound_sender.send(
                    SoundMessage::VolumeChange(volume_slider.channel_name.clone(), volume)
                ).unwrap();
                Command::none()
            }
            
            _ => Command::none()
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let button_tray = Row::new()
                .align_items(Align::Start)
                .spacing(5)
                .push(
                    Button::new(&mut self.gamelog_button, Text::new("Load gamelog.txt").size(16))
                        .on_press(UIMessage::GamelogPressed),
                )
                .push(
                    Button::new(&mut self.soundpack_button, Text::new("Load soundpack").size(16))
                        .on_press(UIMessage::SoundpackPressed),
                )
                .push(
                    Button::new(&mut self.ignore_button, Text::new("Load ignore.txt").size(16))
                        .on_press(UIMessage::IgnorePressed),
                )
                .push(Row::new().width(Length::Fill))
                .push(
                    Button::new(&mut self.about_button, Text::new("About").size(16))
                        .on_press(UIMessage::AboutPressed),
                );
            let scroll = Scrollable::new(&mut self.scroll_state).padding(15).spacing(5);
            let volume_sliders_scroll = self.volumes
                .iter_mut()
                .enumerate()
                .fold(scroll, |scroll, (idx, channel)| {
                    let row = Row::new()
                        .push(
                            Text::new(format!("{}\n{}", channel.channel_name, channel.channel_volume as u32))
                                .size(16)
                                .horizontal_alignment(iced::HorizontalAlignment::Center)
                                .width(Length::Units(60))
                        )
                        .push(
                            Slider::new(
                                &mut channel.channel_slider,
                                0.0..=100.0,
                                channel.channel_volume,
                                move |new_volume| UIMessage::ChannelVolumeChanged(idx, new_volume)
                            )
                        );
                    
                    scroll.push(row)
                });
            let contents = Column::new()
                .padding(5)
                .align_items(Align::Start)
                .push(button_tray)
                .push(volume_sliders_scroll);
            
            Container::new(contents)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
    }
}