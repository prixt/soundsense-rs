use super::*;
use std::sync::mpsc::Receiver;
use std::collections::VecDeque;

/// Struct responsible of playing looping sounds.
pub struct LoopPlayer {
    /// Atomic reference cell to the SourceQueueInput.
    /// Sources are input here to be played.
    queue_tx: Arc<queue::SourcesQueueInput<f32>>,
    /// Whether the loop is stopped.
    /// Playing will cause a new source to be played.
    stopped: Arc<AtomicBool>,
    /// Whether the loop is paused.
    /// Playing will resume the source.
    paused: Arc<AtomicBool>,
    /// Whether current playing sound should be skipped.
    skipped: Arc<AtomicBool>,
    /// LoopPlayer's volume.
    /// This is different from local_volume. This is for dynamic volume changes.
    volume: VolumeLock,
    /// Channel's volume.
    local_volume: VolumeLock,
    /// Total volume (SoundManager's volume).
    total_volume: VolumeLock,
    /// Channel's is_paused.
    local_is_paused: IsPausedLock,
    /// Total is_paused (SoundManager's is_paused).
    total_is_paused: IsPausedLock,
    /// Option for Receiver that checks if the current source has finished playing.
    sleep_until_end: Option<Receiver<()>>,
    /// SoundFile deque.
    /// Whenever a source finishes playing, the first file will play, then the deque rotates.
    files: VecDeque<SoundFile>,
}
impl LoopPlayer {
    #[inline]
    pub fn new(
        device: &Device,
        local_volume: VolumeLock,
        total_volume: VolumeLock,
        local_is_paused: IsPausedLock,
        total_is_paused: IsPausedLock,
    ) -> Self {
        let (queue_tx, queue_rx) = queue::queue(true);
        play_raw(device, queue_rx);
        Self {
            queue_tx,
            local_volume,
            total_volume,
            local_is_paused,
            total_is_paused,
            stopped: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            skipped: Arc::new(AtomicBool::new(false)),
            volume: VolumeLock::new(),
            sleep_until_end: None,
            files: VecDeque::new(),
        }
    }

    #[inline]
    pub fn play(&self) {
        self.stopped.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
    }

    #[inline]
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    #[inline]
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn skip(&self) {
        self.skipped.store(true, Ordering::SeqCst);
    }

    #[inline]
    pub fn set_volume(&self, volume: f32) {
        self.volume.set(volume);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_volume(&self) -> f32 {
        self.volume.get()
    }

    /// Number of sources currently playing. Will always be 0 or 1.
    #[inline]
    pub fn len(&self) -> usize {
        !(self.is_paused() || self.is_stopped() || self.files.is_empty()) as usize
    }

    /// Change the loop.
    /// Replaces the current set of files with another one.
    pub fn change_loop(
        &mut self,
        device: &Device,
        files: &[SoundFile],
        rng: &mut ThreadRng
    ) {
        self.stop();
        self.files = files.iter().cloned().collect();
        let (front, back) = self.files.as_mut_slices();
        front.shuffle(rng); back.shuffle(rng);
        let (queue_tx, queue_rx) = queue::queue(true);
        play_raw(device, queue_rx);
        let volume = self.volume.get();
        self.stopped = Arc::new(AtomicBool::new(false));
        self.paused = Arc::new(AtomicBool::new(false));
        self.volume = VolumeLock::new();
        self.volume.set(volume);
        self.queue_tx = queue_tx;
        self.append_file(rng);
    }

    /// Gets sound source(s) from the first file path, and append to the SourceQueue.
    fn append_file(&mut self, rng: &mut ThreadRng) {
        let file = self.files.front_mut().unwrap();
        let files = match &file.r#type {
            SoundFileType::IsPath(path) => vec![path.clone()],
            SoundFileType::IsPlaylist(paths) => paths.to_vec(),
        };
        let volume = file.volume;
        let balance = if file.random_balance {
            None
        } else {
            Some(file.balance)
        };
        for path in files.iter() {
            let f = match fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    warn!("Failed to open file {}: {}", path.display(), e);
                    warn!("Will ignore this file.");
                    continue
                }
            };
            match Decoder::new(f) {
                Ok(source) => {
                    let balance = balance.unwrap_or_else(||rng.gen_range(-1.0, 1.0));
                    self.append_source(source, volume, balance)
                }
                Err(e) => {
                    warn!("Error while decoding {}: {}", path.display(), e);
                    warn!("Will ignore this source.");
                }
            }
        }
    }

    /// Wraps the source with the appropriate control wrappers, then adds it to the queue.
    fn append_source<S>(&mut self, source: S, source_volume: f32, balance: f32)
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send
    {
        let stopped = self.stopped.clone();
        let paused = self.paused.clone();
        let skipped = self.skipped.clone();
        let volume = self.volume.clone();
        let local_volume = self.local_volume.clone();
        let total_volume = self.total_volume.clone();
        let local_is_paused = self.local_is_paused.clone();
        let total_is_paused = self.total_is_paused.clone();
        let source = source
            .pausable(false)
            .amplify(1.0)
            .stoppable()
            .periodic_access(Duration::from_millis(5),
                move |src| {
                    if stopped.load(Ordering::Relaxed)
                    || skipped.swap(false, Ordering::Relaxed) {
                        src.stop();
                    }
                    else {
                        src.inner_mut()
                            .set_factor(
                                source_volume
                                * volume.get()
                                * local_volume.get()
                                * total_volume.get()
                            );
                        src.inner_mut()
                            .inner_mut()
                            .set_paused(
                                paused.load(Ordering::Relaxed)
                                || local_is_paused.get()
                                || total_is_paused.get()
                            );
                    }
                }
            ).convert_samples::<f32>();
        // If balance is equal, just append it to queue.
        if balance == 0.0 {
            self.sleep_until_end = Some(self.queue_tx.append_with_signal(source));
        }
        // If not, add a Spatial wrapper around the source, then append it to queue.
        else {
            let source = source.buffered();
            let source = Spatial::new(
                source,
                [balance, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            );
            self.sleep_until_end = Some(self.queue_tx.append_with_signal(source));
        }
    }

    /// Maintain the loop.
    pub fn maintain(&mut self, rng: &mut ThreadRng) {
        use std::sync::mpsc::TryRecvError;
        if self.stopped.load(Ordering::Relaxed) {return}
        if let Some(song_end_receiver) = &mut self.sleep_until_end {
            match song_end_receiver.try_recv() {
                Ok(_) => self.on_source_end(rng),
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) =>
                    panic!("TryRecvError::Disconnected on LoopPlayer maintain!"),
            }
        }
    }

    /// Triggerd when the current source ends.
    /// If there are no more sources in queue, rotated the files deque, and appends the first file.
    fn on_source_end(&mut self, rng: &mut ThreadRng) {
        trace!("Song finished.");
        if !self.files.is_empty() && !self.stopped.load(Ordering::Relaxed)
        {
            trace!("  Playing next song.");
            self.files.rotate_left(1);
            self.append_file(rng);
        }
    }
}