#![allow(dead_code, unused_imports)]

use super::*;
use std::sync::{
    Arc,
    Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering}
};

struct Control {
    stopped: AtomicBool,
    count: Arc<AtomicUsize>,
}

pub struct OneshotPlayer {
    volume: Arc<Mutex<f32>>,
    paused: Arc<AtomicBool>,
    controls: Vec<Arc<Control>>,
}

impl OneshotPlayer {
    #[inline]
    pub fn new() -> Self {
        Self {
            volume: Arc::new(Mutex::new(1.0)),
            paused: Arc::new(AtomicBool::new(false)),
            controls: vec![],
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
    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume;
    }

    pub fn add_source<S>(&mut self, device: &Device, source: S, source_volume: f32, _balance: f32)
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send
    {
        let count = Arc::new(AtomicUsize::new(1));
        let control = Arc::new(
            Control {
                stopped: AtomicBool::new(false),
                count,
            }
        );
        let volume = self.volume.clone();
        let paused = self.paused.clone();
        let control_a = control.clone();
        let control_b = control.clone();
        let source = source.convert_samples::<f32>()
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
                                source_volume * (*volume.lock().unwrap())
                            );
                        src.inner_mut()
                            .inner_mut()
                            .set_paused(paused.load(Ordering::SeqCst));
                    }
                }
            ).convert_samples::<f32>();
        // TODO: make Spatial work in here!!
        // let source = Spatial::new(
        //     source,
        //     [_balance, 1.0, 0.0],
        //     [-1.0, 0.0, 0.0],
        //     [1.0, 0.0, 0.0],
        // );
        let source = source::Done::new(source, control_b.count.clone());
        play_raw(device, source);
        self.controls.push(control);
    }

    pub fn maintain(&mut self) {
        self.controls.retain(|c| 
            !c.stopped.load(Ordering::Relaxed)
            && c.count.load(Ordering::Relaxed) == 1
        );
    }
}