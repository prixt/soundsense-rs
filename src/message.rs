/// Messages sent from the UI thread to the Sound thread.
#[non_exhaustive]
pub enum SoundMessage {
    /// Reload the gamelog with this path.
    ChangeGamelog(std::path::PathBuf),
    /// Reload the soundpack with this path.
    ChangeSoundpack(std::path::PathBuf),
    /// Reload the ignore list with this path.
    ChangeIgnoreList(std::path::PathBuf),
    /// Change the volume of a channel.
    /// "total" is total volume, other values are specific channels.
    VolumeChange(Box<str>, f32),
    /// Change the threshold of a channel.
    /// "total" is total threshold, other values are specific channels.
    ThresholdChange(Box<str>, u8),
    /// Skip sound currently played by loop_player
    SkipCurrentSound(Box<str>),
    /// Play/Pause channel
    PlayPause(Box<str>),
    /// Store the current channels volumes to a config file.
    SetCurrentVolumesAsDefault(std::fs::File),
}

/// Message sent from the Sound thread to the UI thread.
#[non_exhaustive]
pub enum UIMessage {
    /// The gamelog finished loading.
    LoadedGamelog,
    /// The soundpack finished loading.
    /// Contains the names of the loaded channels.
    LoadedSoundpack(Vec<Box<str>>),
    /// The ignore list finished loading.
    LoadedIgnoreList,
    /// Loaded the default volumes from config.
    LoadedVolumeSettings(Vec<(Box<str>,f32)>),
    /// The Channel IsPause had been set.
    ChannelWasPlayPaused(Box<str>, bool),
    /// There was an error in the Sound thread.
    SoundThreadPanicked(String,String),
}