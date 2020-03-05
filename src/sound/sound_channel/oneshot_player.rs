#![allow(dead_code)]

use super::*;

/// Struct responsible for one playing oneshot source.
struct Control {
    /// This volume is independent from Channel's local_volume and SoundManager's total_volume. 
    volume: VolumeLock,
    /// Whether the source is stopped.
    stopped: AtomicBool,
    /// Marker to check whether the sound has stopped playing.
    count: Arc<AtomicUsize>,
}

/// Struct responsible of playing oneshot sounds.
pub struct OneshotPlayer {
    /// Whether the oneshot player is paused.
    paused: Arc<AtomicBool>,
    /// Vector of controls, each responsible for a different source.
    controls: Vec<Arc<Control>>,
    /// Channel's local_volume.
    local_volume: VolumeLock,
    /// SoundManager's total_volume.
    total_volume: VolumeLock,
    /// Channel's is_paused.
    local_is_paused: IsPausedLock,
    /// SoundManager's is_paused
    total_is_paused: IsPausedLock,
}

impl OneshotPlayer {
    #[inline]
    pub fn new(
        local_volume: VolumeLock,
        total_volume: VolumeLock,
        local_is_paused: IsPausedLock,
        total_is_paused: IsPausedLock,
    ) -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            controls: vec![],
            local_volume,
            total_volume,
            local_is_paused,
            total_is_paused,
        }
    }

    #[inline]
    pub fn play(&self) {
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

    /// Make all playing sources stop. 
    #[inline]
    pub fn stop(&self) {
        for control in self.controls.iter() {
            control.stopped.store(true, Ordering::SeqCst);
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.controls.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get_volume(&self, idx: usize) -> f32 {
        self.controls[idx].volume.get()
    }

    #[inline]
    pub fn set_volume(&self, idx: usize, volume: f32) {
        self.controls[idx].volume.set(volume);
    }

    /// Add a oneshot source.
    /// Generate a control for the source.
    /// Wraps the source in appropriate control wraps plays it.
    pub fn add_source<S>(
        &mut self,
        device: &Device,
        source: S,
        source_volume: f32,
        balance: f32
    )
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send
    {
        let count = Arc::new(AtomicUsize::new(1));
        let control = Arc::new(
            Control {
                volume: VolumeLock::new(),
                stopped: AtomicBool::new(false),
                count,
            }
        );
        let paused = self.paused.clone();
        let local_volume = self.local_volume.clone();
        let total_volume = self.total_volume.clone();
        let local_is_paused = self.local_is_paused.clone();
        let total_is_paused = self.total_is_paused.clone();
        let control_a = control.clone();
        let control_b = control.clone();
        let source = source
            .pausable(false)
            .amplify(1.0)
            .stoppable()
            .periodic_access(Duration::from_millis(5),
                move |src| {
                    if control_a.stopped.load(Ordering::Relaxed) {
                        src.stop();
                    }
                    else {
                        src.inner_mut()
                            .set_factor(
                                source_volume
                                * control_a.volume.get()
                                * local_volume.get()
                                * total_volume.get()
                                * if local_is_paused.get() {0.0} else {1.0}
                                * if total_is_paused.get() {0.0} else {1.0}
                            );
                        src.inner_mut()
                            .inner_mut()
                            .set_paused(
                                paused.load(Ordering::Relaxed)
                            );
                    }
                }
            ).convert_samples::<f32>();
        let source = source::Done::new(source, control_b.count.clone());
        if balance == 0.0 {
            play_raw(device, source);
        }
        else {
            let source = source.buffered();
            let source = Spatial::new(
                source,
                [balance, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            );
            play_raw(device, source);
        }
        self.controls.push(control);
    }

    /// Remove all controls if stopped, or if the source has finished playing.
    pub fn maintain(&mut self) {
        self.controls.retain(|c| 
            !c.stopped.load(Ordering::Relaxed)
            && c.count.load(Ordering::Relaxed) == 1
        );
    }
}