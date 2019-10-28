use core::sync::atomic::{AtomicBool, AtomicUsize};
use sched::binding::spinlock::SpinlockParamBinding;
use std::sync::Arc;

pub struct PageData {
    pub length: Arc<AtomicUsize>,
    pub gates: Arc<[Arc<AtomicBool>]>,
    pub step_cur: Arc<AtomicUsize>,
    pub clock_mul: Arc<AtomicUsize>,
    pub clock_div: Arc<AtomicUsize>,
    pub probability: Arc<SpinlockParamBinding<f32>>,
    pub volume: Arc<SpinlockParamBinding<f32>>,
    pub volume_rand: Arc<SpinlockParamBinding<f32>>,
}

impl Default for PageData {
    fn default() -> Self {
        Self::new()
    }
}

impl PageData {
    pub fn new() -> Self {
        Self {
            length: Arc::new(AtomicUsize::new(16)),
            step_cur: Arc::new(AtomicUsize::new(0)),
            gates: Arc::new([
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
            ]),
            clock_div: Arc::new(AtomicUsize::new(1)),
            clock_mul: Arc::new(AtomicUsize::new(1)),
            probability: Arc::new(SpinlockParamBinding::new(1f32)),
            volume: Arc::new(SpinlockParamBinding::new(1f32)),
            volume_rand: Arc::new(SpinlockParamBinding::new(0f32)),
        }
    }
}