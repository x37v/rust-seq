use crate::item_source::ItemSource;
use ::spinlock::Mutex;
use std::mem::MaybeUninit;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};

pub struct ChannelItemSource<T> {
    recv: Mutex<Receiver<MaybeUninit<T>>>,
}

pub struct ChannelItemCreator<T> {
    send: SyncSender<MaybeUninit<T>>,
}

impl<T> ChannelItemSource<T> {
    fn new(r: Receiver<MaybeUninit<T>>) -> Self {
        Self {
            recv: Mutex::new(r),
        }
    }
}

impl<T> ItemSource<T, Box<T>> for ChannelItemSource<T> {
    /// Try to get an item and set it to `init`
    /// Passes back a `Err(init)` on failure.
    fn try_get(&mut self, init: T) -> Result<Box<T>, T> {
        if let Some(mut item) = self.recv.lock().try_recv().ok() {
            unsafe {
                let ptr = item.as_mut_ptr();
                ptr.write(init);
                Ok(Box::from_raw(ptr))
            }
        } else {
            Err(init)
        }
    }
}

impl<T> ChannelItemCreator<T> {
    fn new(send: SyncSender<MaybeUninit<T>>) -> Self {
        Self { send }
    }

    pub fn fill(&mut self) -> Result<(), TrySendError<()>> {
        loop {
            match self.send.try_send(MaybeUninit::uninit()) {
                Ok(()) => continue,
                Err(TrySendError::Full(_)) => return Ok(()),
                Err(TrySendError::Disconnected(_)) => return Err(TrySendError::Disconnected(())),
            }
        }
    }
}

pub fn item_source<T>(n: usize) -> (ChannelItemCreator<T>, ChannelItemSource<T>) {
    let (s, r) = sync_channel(n);
    let mut c = ChannelItemCreator::new(s);
    c.fill().expect("failed to fill");
    (c, ChannelItemSource::new(r))
}
