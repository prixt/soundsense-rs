pub enum SoundMessage {
	GlibSender(glib::Sender<UIMessage>),
	ChangeGamelog(std::path::PathBuf),
	ChangeSoundpack(std::path::PathBuf),
	VolumeChange(Box<str>, f32),
}

pub enum UIMessage {
	ChannelNames(Vec<Box<str>>),
	ChannelChangeSong(Box<str>, Box<str>),
}