pub enum SoundMessage {
	HandlerInit(crate::ui::UIHandle),
	ChangeGamelog(std::path::PathBuf),
	ChangeSoundpack(std::path::PathBuf),
	VolumeChange(Box<str>, f32),
}

#[derive(Deserialize)]
pub struct VolumeChange {
	pub channel: Box<str>,
	pub volume: f32
}