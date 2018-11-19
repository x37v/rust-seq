use base::{LList, LNode};
use binding::ParamBindingGet;
use binding::ParamBindingSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::mpsc::SyncSender;

static ID_COUNT: AtomicUsize = AtomicUsize::new(0);

pub type ObserverNode = Box<LNode<SyncSender<ObservableId>>>;
pub type ObserverList = LList<SyncSender<ObservableId>>;

trait Observable {
    fn id(&self) -> ObservableId;
    fn add_observer(&self, observer_node: ObserverNode);
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ObservableId(usize);

pub struct ObservableData {
    id: ObservableId,
    observers: spinlock::Mutex<ObserverList>,
}

pub struct ObservableBinding<B, T> {
    binding: T,
    _phantom: PhantomData<AtomicPtr<Box<B>>>, //XXX used atomic so we can share across threads, could have been mutex..
    observer_data: ObservableData,
}

pub fn new_observer_node(sender: SyncSender<ObservableId>) -> ObserverNode {
    LNode::new_boxed(sender)
}

impl ObservableId {
    fn new() -> Self {
        ObservableId(ID_COUNT.fetch_add(1, Ordering::Relaxed))
    }
}

impl ObservableData {
    pub fn new() -> Self {
        Self {
            id: ObservableId::new(),
            observers: spinlock::Mutex::new(LList::new()),
        }
    }

    pub fn notify(&self) {
        let l = self.observers.lock();
        for c in l.iter() {
            let _ = c.try_send(self.id);
        }
    }
}

impl Observable for ObservableData {
    fn id(&self) -> ObservableId {
        self.id
    }
    fn add_observer(&self, observer_node: ObserverNode) {
        let mut l = self.observers.lock();
        l.push_back(observer_node);
    }
}

impl<B, T> ObservableBinding<B, T>
where
    B: Send + Copy,
{
    pub fn new(binding: T) -> Self {
        Self {
            binding,
            _phantom: Default::default(),
            observer_data: ObservableData::new(),
        }
    }

    fn notify(&self) {
        self.observer_data.notify();
    }
}

impl<B, T> Observable for ObservableBinding<B, T> {
    fn id(&self) -> ObservableId {
        self.observer_data.id()
    }
    fn add_observer(&self, observer_node: ObserverNode) {
        self.observer_data.add_observer(observer_node);
    }
}

impl<B, T> ParamBindingSet<B> for ObservableBinding<B, T>
where
    B: Copy + Send,
    T: ParamBindingSet<B>,
{
    fn set(&self, value: B) {
        self.binding.set(value);
        self.notify();
    }
}

impl<B, T> ParamBindingGet<B> for ObservableBinding<B, T>
where
    B: Copy + Send,
    T: ParamBindingGet<B>,
{
    fn get(&self) -> B {
        self.binding.get()
    }
}

impl<B, T> Deref for ObservableBinding<B, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.binding
    }
}

pub mod bpm {
    pub struct ObservableClockData {
        clock_data: ::binding::bpm::ClockData,
        observer_data: super::ObservableData,
    }

    impl ObservableClockData {
        pub fn new(clock_data: ::binding::bpm::ClockData) -> Self {
            Self {
                clock_data,
                observer_data: super::ObservableData::new(),
            }
        }

        fn notify(&self) {
            self.observer_data.notify();
        }
    }

    impl super::Observable for ObservableClockData {
        fn id(&self) -> super::ObservableId {
            self.observer_data.id()
        }
        fn add_observer(&self, observer_node: super::ObserverNode) {
            self.observer_data.add_observer(observer_node);
        }
    }

    impl ::binding::bpm::Clock for ObservableClockData {
        fn bpm(&self) -> f32 {
            self.clock_data.bpm()
        }
        fn set_bpm(&mut self, bpm: f32) {
            self.clock_data.set_bpm(bpm);
            self.notify();
        }

        fn period_micros(&self) -> f32 {
            self.clock_data.period_micros()
        }
        fn set_period_micros(&mut self, period_micros: f32) {
            self.clock_data.set_period_micros(period_micros);
            self.notify();
        }

        fn ppq(&self) -> usize {
            self.clock_data.ppq()
        }
        fn set_ppq(&mut self, ppq: usize) {
            self.clock_data.set_ppq(ppq);
            self.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binding::bpm::Clock;
    use binding::{ParamBindingSet, SpinlockParamBinding};
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
    use std::sync::Arc;

    use std::thread;

    #[test]
    fn id_expectation() {
        let mut u: ObservableBinding<usize, _> = ObservableBinding::new(AtomicUsize::new(2));
        let mut id = u.id().0;

        u = ObservableBinding::new(AtomicUsize::new(3));
        assert_eq!(3, u.get());
        assert!(u.id().0 > id);
        id = u.id().0;

        let i: ObservableBinding<isize, _> = ObservableBinding::new(AtomicIsize::new(7));
        assert!(i.id().0 > id);
        id = i.id().0;

        let b: ObservableBinding<bool, _> =
            ObservableBinding::new(SpinlockParamBinding::new(false));
        assert!(b.id().0 > id);

        assert_eq!(false, b.get());
        id = b.id().0;

        b.set(true);
        assert_eq!(id, b.id().0);
        assert_eq!(true, b.get());

        let child = thread::spawn(move || {
            let mut u: ObservableBinding<usize, _> = ObservableBinding::new(AtomicUsize::new(2));
            assert!(u.id().0 > id);
            id = u.id().0;

            u = ObservableBinding::new(AtomicUsize::new(3));
            assert!(u.id().0 > id);
            id = u.id().0;

            let i: ObservableBinding<isize, _> = ObservableBinding::new(AtomicIsize::new(7));
            assert!(i.id().0 > id);
            assert_eq!(7, i.get());
            id = i.id().0;

            let b: ObservableBinding<bool, _> =
                ObservableBinding::new(SpinlockParamBinding::new(false));
            assert!(b.id().0 > id);
            id = b.id().0;
        });
        assert!(child.join().is_ok());

        u = ObservableBinding::new(AtomicUsize::new(3));
        assert!(u.id().0 > id);
    }

    #[test]
    fn deref() {
        let u: ObservableBinding<usize, _> = ObservableBinding::new(AtomicUsize::new(2));
        assert_eq!(2, u.deref().get());
        assert_eq!(2, u.get());

        u.set(5);
        assert_eq!(5, u.deref().get());
        assert_eq!(5, u.get());

        u.deref().set(23);
        assert_eq!(23, u.deref().get());
        assert_eq!(23, u.get());

        assert_eq!(23, (*u).get());
        assert_eq!(23, u.get());

        (*u).set(42);
        assert_eq!(42, (*u).get());
        assert_eq!(42, u.get());
    }

    #[test]
    fn observe() {
        let (s1, r1) = sync_channel(16);

        let mut u: ObservableBinding<usize, _> = ObservableBinding::new(AtomicUsize::new(2));
        let id = u.id();

        assert!(r1.try_recv().is_err());

        let o = new_observer_node(s1);
        u.add_observer(o);
        assert!(r1.try_recv().is_err());

        //gets one notification
        u.set(23);
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(23, u.get());

        u.set(11);
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(11, u.get());

        let (s2, r2) = sync_channel(16);
        assert!(r1.try_recv().is_err());
        assert!(r2.try_recv().is_err());
        assert_eq!(11, u.get());

        u.set(11);
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert!(r2.try_recv().is_err());
        assert_eq!(11, u.get());

        u.add_observer(new_observer_node(s2));

        u.set(80);
        assert_eq!(id, r1.try_recv().unwrap());
        assert_eq!(id, r2.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert!(r2.try_recv().is_err());
        assert_eq!(80, u.get());

        //deref doesn't signal
        u.deref().set(800);
        assert!(r1.try_recv().is_err());
        assert!(r2.try_recv().is_err());
        assert_eq!(800, u.get());
    }

    #[test]
    fn observe_arc() {
        let (s1, r1) = sync_channel(16);

        let mut u: Arc<ObservableBinding<usize, _>> =
            Arc::new(ObservableBinding::new(AtomicUsize::new(2)));
        let id = u.id();
        assert!(r1.try_recv().is_err());

        let o = new_observer_node(s1);
        u.add_observer(o);

        u.set(2);
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(2, u.get());

        let c = u.clone();
        c.set(40);

        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(40, u.get());
        assert_eq!(40, c.get());
    }

    #[test]
    fn bpm() {
        let (s1, r1) = sync_channel(16);
        let b = Arc::new(spinlock::Mutex::new(bpm::ObservableClockData::new(
            ::binding::bpm::ClockData::new(120.0, 96),
        )));
        let id = b.lock().id();
        assert!(r1.try_recv().is_err());

        let bpm = Arc::new(::binding::bpm::ClockBPMBinding(b.clone()));
        let ppq = Arc::new(::binding::bpm::ClockPPQBinding(b.clone()));
        let micros = Arc::new(::binding::bpm::ClockPeriodMicroBinding(b.clone()));
        let micros2 = micros.clone();

        let c = b.clone();
        assert_eq!(5208f32, c.lock().period_micros().floor());
        assert_eq!(5208f32, micros.get().floor());
        assert_eq!(120f32, c.lock().bpm());
        assert_eq!(120f32, bpm.get());
        assert_eq!(96, c.lock().ppq());
        assert_eq!(96, ppq.get());

        let o = new_observer_node(s1);
        b.lock().add_observer(o);

        bpm.set(2.0);
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(2f32, bpm.get());

        bpm.set(1.0);
        ppq.set(960);
        assert_eq!(id, r1.try_recv().unwrap());
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(1f32, c.lock().bpm());
        assert_eq!(1f32, bpm.get());
        assert_eq!(960, c.lock().ppq());
        assert_eq!(960, ppq.get());

        ppq.set(9600);
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
        assert_eq!(9600, c.lock().ppq());
        assert_eq!(9600, ppq.get());

        micros2.set(5_208.333333f32);
        assert_eq!(5208f32, c.lock().period_micros().floor());
        assert_eq!(5208f32, micros.get().floor());
        assert_eq!(id, r1.try_recv().unwrap());
        assert!(r1.try_recv().is_err());
    }
}
