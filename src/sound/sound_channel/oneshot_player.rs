#![allow(dead_code)]

use super::*;

struct Control {
    volume: VolumeLock,
    stopped: AtomicBool,
    count: Arc<AtomicUsize>,
}

pub struct OneshotPlayer {
    paused: Arc<AtomicBool>,
    controls: Vec<Arc<Control>>,
    local_volume: VolumeLock,
    total_volume: VolumeLock,
}

impl OneshotPlayer {
    #[inline]
    pub fn new(local_volume: VolumeLock, total_volume: VolumeLock) -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            controls: vec![],
            local_volume,
            total_volume,
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
    pub fn is_paused(&self) {
        self.paused.load(Ordering::Relaxed);
    }

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
        let control_a = control.clone();
        let control_b = control.clone();
        let source = source
            .pausable(false)
            .amplify(1.0)
            .stoppable()
            .periodic_access(Duration::from_millis(5),
                move |src| {
                    if control_a.stopped.load(Ordering::SeqCst) {
                        src.stop();
                    }
                    else {
                        src.inner_mut()
                            .set_factor(
                                source_volume
                                * control_a.volume.get()
                                * local_volume.get()
                                * total_volume.get()
                            );
                        src.inner_mut()
                            .inner_mut()
                            .set_paused(paused.load(Ordering::SeqCst));
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

    pub fn maintain(&mut self) {
        self.controls.retain(|c| 
            !c.stopped.load(Ordering::Relaxed)
            && c.count.load(Ordering::Relaxed) == 1
        );
    }
}