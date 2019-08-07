pub enum SoundMessage {
	GlibSender(glib::Sender<UIMessage>),
	ChangeGamelog(std::path::PathBuf),
	ChangeSoundpack(std::path::PathBuf),
	VolumeChange(String, f32),
}

pub enum UIMessage {
	ChannelNames(Vec<String>),
	ChannelChangeSong(String, String),
}