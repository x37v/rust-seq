use base::{LList, LNode};
use binding::ParamBindingGet;
use binding::ParamBindingSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::mpsc::SyncSender;

pub type ObserverNode = Box<LNode<SyncSender<ObservableId>>>;
pub type ObserverList = LList<SyncSender<ObservableId>>;

static ID_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ObservableId(usize);

impl ObservableId {
    fn new() -> Self {
        ObservableId(ID_COUNT.fetch_add(1, Ordering::Relaxed))
    }
}

pub fn new_observer_node(sender: SyncSender<ObservableId>) -> ObserverNode {
    LNode::new_boxed(sender)
}

pub struct ObservableBinding<B, T> {
    id: ObservableId,
    binding: T,
    observers: spinlock::Mutex<ObserverList>,
    _phantom: PhantomData<AtomicPtr<Box<B>>>, //XXX used atomic so we can share across threads, could have been mutex..
}

impl<B, T> ObservableBinding<B, T>
where
    B: Send + Copy,
{
    pub fn new(binding: T) -> Self {
        Self {
            id: ObservableId::new(),
            binding,
            observers: spinlock::Mutex::new(LList::new()),
            _phantom: Default::default(),
        }
    }

    pub fn add_observer(&mut self, observer_node: ObserverNode) {
        let mut l = self.observers.lock();
        l.push_back(observer_node);
    }

    fn notify(&self) {
        let l = self.observers.lock();
        for c in l.iter() {
            let _ = c.try_send(self.id);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use binding::{ParamBindingSet, SpinlockParamBinding};
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

    use std::thread;

    #[test]
    fn id_expectation() {
        let mut u: ObservableBinding<usize, _> = ObservableBinding::new(AtomicUsize::new(2));
        let mut id = u.id.0;

        u = ObservableBinding::new(AtomicUsize::new(3));
        assert_eq!(3, u.get());
        assert!(u.id.0 > id);
        id = u.id.0;

        let i: ObservableBinding<isize, _> = ObservableBinding::new(AtomicIsize::new(7));
        assert!(i.id.0 > id);
        id = i.id.0;

        let b: ObservableBinding<bool, _> =
            ObservableBinding::new(SpinlockParamBinding::new(false));
        assert!(b.id.0 > id);

        assert_eq!(false, b.get());
        id = b.id.0;

        b.set(true);
        assert_eq!(id, b.id.0);
        assert_eq!(true, b.get());

        let child = thread::spawn(move || {
            let mut u: ObservableBinding<usize, _> = ObservableBinding::new(AtomicUsize::new(2));
            assert!(u.id.0 > id);
            id = u.id.0;

            u = ObservableBinding::new(AtomicUsize::new(3));
            assert!(u.id.0 > id);
            id = u.id.0;

            let i: ObservableBinding<isize, _> = ObservableBinding::new(AtomicIsize::new(7));
            assert!(i.id.0 > id);
            assert_eq!(7, i.get());
            id = i.id.0;

            let b: ObservableBinding<bool, _> =
                ObservableBinding::new(SpinlockParamBinding::new(false));
            assert!(b.id.0 > id);
            id = b.id.0;
        });
        assert!(child.join().is_ok());

        u = ObservableBinding::new(AtomicUsize::new(3));
        assert!(u.id.0 > id);
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
        let id = u.id;

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
}
