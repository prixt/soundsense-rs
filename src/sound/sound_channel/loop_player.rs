use super::*;

struct Control {
    stopped: AtomicBool,
    paused: AtomicBool,
    volume: Mutex<f32>,
}

pub struct LoopPlayer {
    queue_tx: Arc<queue::SourcesQueueInput<f32>>,
    in_queue: Arc<AtomicUsize>,
    controls: Arc<Control>,
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
        let control = Control {
            stopped: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            volume: Mutex::new(1.0),
        };
        Self {
            queue_tx,
            in_queue: Arc::new(AtomicUsize::new(0)),
            controls: Arc::new(control),
            local_volume,
            total_volume,
            sleep_until_end: None,
            current_file_idx: 0,
            files: vec![],
        }
    }

    #[inline]
    pub fn play(&self) {
        self.controls.stopped.store(false, Ordering::SeqCst);
        self.controls.paused.store(false, Ordering::SeqCst);
    }

    #[inline]
    pub fn pause(&self) {
        self.controls.paused.store(true, Ordering::SeqCst);
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.controls.paused.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn stop(&self) {
        self.controls.stopped.store(true, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn set_volume(&self, volume: f32) {
        *self.controls.volume.lock().unwrap() = volume;
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
        let volume = *self.controls.volume.lock().unwrap();
        let controls = Control {
            stopped: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            volume: Mutex::new(volume),
        };
        self.controls = Arc::new(controls);
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
            let f = fs::File::open(path).unwrap();
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
        let controls = self.controls.clone();
        let local_volume = self.local_volume.clone();
        let total_volume = self.total_volume.clone();
        let source = source
            .pausable(false)
            .amplify(1.0)
            .stoppable()
            .periodic_access(Duration::from_millis(5),
                move |src| {
                    if controls.stopped.load(Ordering::SeqCst) {
                        src.stop();
                    }
                    else {
                        src.inner_mut()
                            .set_factor(
                                source_volume
                                * (*controls.volume.lock().unwrap())
                                * local_volume.get()
                                * total_volume.get()
                            );
                        src.inner_mut()
                            .inner_mut()
                            .set_paused(controls.paused.load(Ordering::SeqCst));
                    }
                }
            ).convert_samples::<f32>();
        // TODO: make Spatial work in here!!
        #[cfg(not(taget_os="windows"))]
        let source = Spatial::new(
            source,
            [balance, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        );
        self.in_queue.fetch_add(1, Ordering::SeqCst);
        let source = source::Done::new(source, self.in_queue.clone());
        self.sleep_until_end = Some(self.queue_tx.append_with_signal(source));
    }

    pub fn maintain(&mut self, rng: &mut ThreadRng) {
        use std::sync::mpsc::TryRecvError;
        if self.controls.stopped.load(Ordering::Relaxed) {return}
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
        if self.in_queue.load(Ordering::Relaxed) == 0 {
            self.current_file_idx += 1;
            if self.files.len() == self.current_file_idx {
                self.current_file_idx = 0;
            }
            self.append_file(self.current_file_idx, rng);
        }
    }
}