use std::sync::Arc;

pub type TimePoint = isize;
pub type SeqFn = Arc<SchedCall>;

pub trait SchedCall {
    fn sched_call(&mut self, &mut Sched) -> Option<TimePoint>;
}

pub trait Sched {
    fn schedule(&mut self, t: TimePoint, f: SeqFn);
}

impl<F: Fn(&mut Sched) -> Option<TimePoint>> SchedCall for F {
    fn sched_call(&mut self, s: &mut Sched) -> Option<TimePoint> {
        (*self)(s)
    }
}

pub trait SeqCached<T> {
    fn pop() -> Option<Arc<T>>;
    fn push(v: Arc<T>) -> ();
}

pub struct Seq {
    items: Vec<SeqFn>,
}

impl Sched for Seq {
    fn schedule(&mut self, _t: TimePoint, f: SeqFn) {
        self.items.push(f);
    }
}

impl Seq {
    pub fn new() -> Self {
        Seq { items: Vec::new() }
    }

    pub fn run(&mut self) {
        println!("run!");
        let l: Vec<SeqFn> = self.items.drain(..).collect();
        for mut f in l {
            if let Some(fm) = Arc::get_mut(&mut f) {
                if let Some(_n) = fm.sched_call(self) {
                    self.items.push(f);
                }
            }
        }
    }
}

