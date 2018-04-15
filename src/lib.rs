extern crate xnor_llist;

use std::sync::Arc;

pub type ITimePoint = isize;
pub type UTimePoint = usize;
pub type SeqFn = Arc<SchedCall>;

pub struct TimedFn {
    time: ITimePoint,
    func: SeqFn,
}

impl TimedFn {
    pub fn new(func: SeqFn) -> Self {
        TimedFn { func, time: 0 }
    }
}

pub type SeqFnNode = Box<xnor_llist::Node<TimedFn>>;

#[macro_export]
macro_rules! boxed_fn {
    ($x:expr) => {
        xnor_llist::Node::new_boxed(TimedFn::new(Arc::new($x)))
    }
}

//XXX is it cool to just say that these shits are Sync and send?
pub trait SchedCall: Sync + Send {
    fn sched_call(&mut self, &mut Sched) -> Option<UTimePoint>;
}

pub trait Sched {
    fn schedule(&mut self, t: ITimePoint, n: SeqFnNode);
}

impl<F: Fn(&mut Sched) -> Option<UTimePoint>> SchedCall for F
where
    F: Sync + Send,
{
    fn sched_call(&mut self, s: &mut Sched) -> Option<UTimePoint> {
        (*self)(s)
    }
}

impl SchedCall for TimedFn {
    fn sched_call(&mut self, s: &mut Sched) -> Option<UTimePoint> {
        if let Some(f) = Arc::get_mut(&mut self.func) {
            f.sched_call(s)
        } else {
            None
        }
    }
}

pub trait SeqCached<T> {
    fn pop() -> Option<Arc<T>>;
    fn push(v: Arc<T>) -> ();
}

pub struct Seq {
    list: xnor_llist::List<TimedFn>,
}

impl Sched for Seq {
    fn schedule(&mut self, t: ITimePoint, mut f: SeqFnNode) {
        f.time = t;
        self.list.insert(f, |n, o| n.time <= o.time);
    }
}

impl Seq {
    pub fn new() -> Self {
        Seq {
            list: xnor_llist::List::new(),
        }
    }

    pub fn run(&mut self) {
        let mut reschedule = xnor_llist::List::new();
        while let Some(mut timedfn) = self.list.pop_front() {
            if let Some(t) = timedfn.sched_call(self) {
                timedfn.time = t as ITimePoint + timedfn.time;
                reschedule.push_back(timedfn);
            }
        }
        for n in reschedule.into_iter() {
            self.schedule(n.time, n);
        }
    }
}
