pub enum SoundMessage {
    ChangeGamelog(std::path::PathBuf),
    ChangeSoundpack(std::path::PathBuf),
    ChangeIgnoreList(std::path::PathBuf),
    VolumeChange(Box<str>, f32),
    SetCurrentPathsAsDefault,
    SetCurrentVolumesAsDefault,
}

pub enum UIMessage {
    LoadedGamelog,
    LoadedSoundpack(Vec<Box<str>>),
    LoadedIgnoreList,
}