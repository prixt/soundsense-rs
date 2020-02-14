#[non_exhaustive]
pub enum SoundMessage {
    ChangeGamelog(std::path::PathBuf),
    ChangeSoundpack(std::path::PathBuf),
    ChangeIgnoreList(std::path::PathBuf),
    VolumeChange(Box<str>, f32),
    // SetCurrentPathsAsDefault,
    SetCurrentVolumesAsDefault(std::fs::File),
}

#[allow(dead_code)]
#[non_exhaustive]
pub enum UIMessage {
    LoadedGamelog,
    LoadedSoundpack(Vec<Box<str>>),
    LoadedIgnoreList,
    LoadedVolumeSettings(Vec<(Box<str>,f32)>),
}