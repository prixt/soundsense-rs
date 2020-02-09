pub enum SoundMessage {
    ChangeGamelog(std::path::PathBuf),
    ChangeSoundpack(std::path::PathBuf),
    ChangeIgnoreList(std::path::PathBuf),
    VolumeChange(String, f32),
    SetCurrentPathsAsDefault,
    SetCurrentVolumesAsDefault,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum UIMessage {
    GamelogPressed,

    SoundpackPressed,
    SoundpackLoaded,

    IgnorePressed,

    Resolved,

    AboutPressed,
    ChannelVolumeChanged(usize, f32),
}