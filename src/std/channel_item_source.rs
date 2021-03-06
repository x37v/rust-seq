use crate::item_source::ItemSource;
use spin::Mutex;
use std::mem::MaybeUninit;
use std::ops::DerefMut;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};

pub struct ChannelItemSource<T>
where
    T: Send,
{
    recv: Mutex<Receiver<Box<MaybeUninit<T>>>>,
}

pub struct ChannelItemCreator<T>
where
    T: Send,
{
    send: SyncSender<Box<MaybeUninit<T>>>,
}

impl<T> ChannelItemSource<T>
where
    T: Send,
{
    fn new(r: Receiver<Box<MaybeUninit<T>>>) -> Self {
        Self {
            recv: Mutex::new(r),
        }
    }
}

impl<T> ItemSource<T, Box<T>> for ChannelItemSource<T>
where
    T: Send,
{
    /// Try to get an item and set it to `init`
    /// Passes back a `Err(init)` on failure.
    fn try_get(&self, init: T) -> Result<Box<T>, T> {
        if let Some(mut item) = self.recv.lock().try_recv().ok() {
            unsafe {
                item.deref_mut().as_mut_ptr().write(init);
                Ok(std::mem::transmute(item))
            }
        } else {
            Err(init)
        }
    }
}

impl<T> ChannelItemCreator<T>
where
    T: Send,
{
    fn new(send: SyncSender<Box<MaybeUninit<T>>>) -> Self {
        Self { send }
    }

    pub fn fill(&mut self) -> Result<(), TrySendError<()>> {
        loop {
            match self.send.try_send(Box::new(MaybeUninit::uninit())) {
                Ok(()) => continue,
                Err(TrySendError::Full(_)) => return Ok(()),
                Err(TrySendError::Disconnected(_)) => return Err(TrySendError::Disconnected(())),
            }
        }
    }
}

pub fn item_source<T>(n: usize) -> (ChannelItemCreator<T>, ChannelItemSource<T>)
where
    T: Send,
{
    let (s, r) = sync_channel(n);
    let mut c = ChannelItemCreator::new(s);
    c.fill().expect("failed to fill");
    (c, ChannelItemSource::new(r))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get() {
        let (mut c, s): (_, ChannelItemSource<usize>) = item_source(2);
        assert_eq!(Ok(()), c.fill());

        let x = s.try_get(23usize);
        assert!(x.is_ok());
        assert_eq!(23usize, *x.unwrap());

        let x = s.try_get(42usize);
        assert!(x.is_ok());
        assert_eq!(42usize, *x.unwrap());

        let x = s.try_get(2usize);
        assert_eq!(Err(2usize), x);

        assert_eq!(Ok(()), c.fill());

        let x = s.try_get(2usize);
        assert!(x.is_ok());
        assert_eq!(2usize, *x.unwrap());

        assert_eq!(Ok(()), c.fill());

        let x = s.try_get(1usize);
        assert!(x.is_ok());
        assert_eq!(1usize, *x.unwrap());

        let x = s.try_get(2usize);
        assert!(x.is_ok());
        assert_eq!(2usize, *x.unwrap());

        let x = s.try_get(3usize);
        assert_eq!(Err(3usize), x);
    }
}
