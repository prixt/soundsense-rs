use super::*;

pub struct LoopPlayer {
    queue_tx: Arc<queue::SourcesQueueInput<f32>>,
    in_queue: usize,
    stopped: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    volume: VolumeLock,
    local_volume: VolumeLock,
    total_volume: VolumeLock,
    sleep_until_end: Option<Receiver<()>>,
    current_file_idx: usize,
    files: Vec<SoundFile>,
}
impl LoopPlayer {
    #[inline]
    pub fn new(
        device: &Device,
        local_volume: VolumeLock,
        total_volume: VolumeLock
    ) -> Self {
        let (queue_tx, queue_rx) = queue::queue(true);
        play_raw(device, queue_rx);
        Self {
            queue_tx,
            in_queue: 0,
            local_volume,
            total_volume,
            stopped: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            volume: VolumeLock::new(),
            sleep_until_end: None,
            current_file_idx: 0,
            files: vec![],
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
    pub fn set_volume(&self, volume: f32) {
        self.volume.set(volume);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_volume(&self) -> f32 {
        self.volume.get()
    }

    #[inline]
    pub fn len(&self) -> usize {
        (!self.is_paused() && !self.is_stopped() && self.in_queue != 0) as usize
    }

    pub fn change_loop(
        &mut self,
        device: &Device,
        files: &[SoundFile],
        rng: &mut ThreadRng
    ) {
        self.stop();
        self.current_file_idx = 0;
        self.files.clear(); self.files.extend_from_slice(files);
        let (queue_tx, queue_rx) = queue::queue(true);
        play_raw(device, queue_rx);
        let volume = self.volume.get();
        self.stopped = Arc::new(AtomicBool::new(false));
        self.paused = Arc::new(AtomicBool::new(false));
        self.volume = VolumeLock::new();
        self.volume.set(volume);
        self.in_queue = 0;
        self.queue_tx = queue_tx;
        self.append_file(0, rng);
    }

    fn append_file(&mut self, idx: usize, rng: &mut ThreadRng) {
        let file = self.files.get_mut(idx).unwrap();
        let files = match &file.r#type {
            SoundFileType::IsPath(path) =>
                vec![path.clone()],
            SoundFileType::IsPlaylist(paths) => 
                paths.iter().map(PathBuf::from).collect(),
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
                Err(e) => panic!("Failed to open file {}\nError: {}", path.display(), e),
            };
            let source = Decoder::new(f);
            match source {
                Ok(source) => {
                    let balance = balance.unwrap_or_else(||rng.gen_range(-1.0, 1.0));
                    self.append_source(source, volume, balance)
                }
                Err(e) => 
                    eprintln!("Error while asserting {}: {}", path.display(), e),
            }
        }
    }

    fn append_source<S>(&mut self, source: S, source_volume: f32, balance: f32)
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send
    {
        let stopped = self.stopped.clone();
        let paused = self.paused.clone();
        let volume = self.volume.clone();
        let local_volume = self.local_volume.clone();
        let total_volume = self.total_volume.clone();
        let source = source
            .pausable(false)
            .amplify(1.0)
            .stoppable()
            .periodic_access(Duration::from_millis(5),
                move |src| {
                    if stopped.load(Ordering::SeqCst) {
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
                            .set_paused(paused.load(Ordering::SeqCst));
                    }
                }
            ).convert_samples::<f32>();
        self.in_queue += 1;
        if balance == 0.0 {
            self.sleep_until_end = Some(self.queue_tx.append_with_signal(source));
        }
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

    fn on_source_end(&mut self, rng: &mut ThreadRng) {
        self.in_queue -= 1;
        if self.in_queue == 0 && !self.stopped.load(Ordering::Relaxed)
        {
            self.current_file_idx += 1;
            if self.files.len() == self.current_file_idx {
                self.current_file_idx = 0;
            }
            self.append_file(self.current_file_idx, rng);
        }
    }
}