use std::marker::PhantomData;
use base::{LList, LNode};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use binding::ParamBindingSet;

type ObserverNode = LNode<SyncSender<ObservableId>>;
type ObserverList = LList<SyncSender<ObservableId>>;

static ID_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct ObservableId(usize);

pub struct ObservableBindingSet<B, T> {
    id: ObservableId,
    binding: T,
    observers: spinlock::Mutex<ObserverList>,
    _phantom: PhantomData<AtomicPtr<Box<B>>>, //XXX used atomic so we can share across threads, could have been mutex..
}

impl<B, T> ObservableBindingSet<B, T>
where T: ParamBindingSet<B>
{
    pub fn new(binding: T) -> Self {
        Self {
            id: ObservableId(ID_COUNT.fetch_add(1, Ordering::Relaxed)),
            binding,
            observers: spinlock::Mutex::new(LList::new()),
            _phantom: Default::default(),
        }
    }

     fn notify(&self) {
        let g = self.observers.lock();
        for c in g.iter() {
            let _ = c.try_send(self.id);
        }
    }
}

impl<B, T> ParamBindingSet<B> for ObservableBindingSet<B, T>
where B: Copy + Send,
      T: ParamBindingSet<B>
{
    fn set(&self, value: B) {
        self.binding.set(value);
        self.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::thread;
    use binding::{ParamBindingSet, SpinlockParamBinding};

    #[test]
    fn id_expectation() {
        //we would never reset the ids but in these unit tests we want to make sure we start at 0
        ID_COUNT.store(0, Ordering::SeqCst);
        let mut u = ObservableBindingSet::new(AtomicUsize::new(2));
        assert_eq!(0, u.id.0);

        u = ObservableBindingSet::new(AtomicUsize::new(3));
        assert_eq!(1, u.id.0);
        
        let i = ObservableBindingSet::new(AtomicIsize::new(7));
        assert_eq!(2, i.id.0);

        let b = ObservableBindingSet::new(SpinlockParamBinding::new(false));
        assert_eq!(3, b.id.0);

        let child = thread::spawn(move || {
            let mut u = ObservableBindingSet::new(AtomicUsize::new(2));
            assert_eq!(4, u.id.0);

            u = ObservableBindingSet::new(AtomicUsize::new(3));
            assert_eq!(5, u.id.0);
            
            let i = ObservableBindingSet::new(AtomicIsize::new(7));
            assert_eq!(6, i.id.0);

            let b = ObservableBindingSet::new(SpinlockParamBinding::new(false));
            assert_eq!(7, b.id.0);
        });
        assert!(child.join().is_ok());

        u = ObservableBindingSet::new(AtomicUsize::new(3));
        assert_eq!(8, u.id.0);
    }

    #[test]
    fn id_expectation2() {
        //we would never reset the ids but in these unit tests we want to make sure we start at 0
        ID_COUNT.store(0, Ordering::SeqCst);
        let mut u = ObservableBindingSet::new(AtomicUsize::new(2));
        assert_eq!(0, u.id.0);
    }
}
