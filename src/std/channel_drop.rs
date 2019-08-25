use ::spinlock::Mutex;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};

lazy_static::lazy_static! {
    static ref DROP_CHANNEL: (
        Mutex<SyncSender<Box<dyn Send>>>,
        Mutex<Option<Receiver<Box<dyn Send>>>>) = {
        let (s, r) = sync_channel(1024);
        (Mutex::new(s), Mutex::new(Some(r)))
    };
}

/// Get the consume function (if nothing already got it).
///
/// You must hold onto this until the end of the program and keep running
/// it periodically for the consume operation to work.
///
/// Will return an `TrySendError::Disconnected` if it gets disconnected
/// `Ok(())` otherwise.
///
/// TODO: could use a struct to hold this and when it drops, return the Receiver.
pub fn get_consume() -> Option<Box<impl Fn() -> Result<(), TryRecvError>>> {
    let r = DROP_CHANNEL.1.lock().take();
    if let Some(rec) = r {
        Some(Box::new(move || {
            loop {
                match rec.try_recv() {
                    Ok(_) => continue,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return Err(TryRecvError::Disconnected),
                }
            }
            Ok(())
        }))
    } else {
        None
    }
}

/// A wrapper for a Boxed item that when dropped, will push its boxed item to a channel so that it
/// can be cleaned up in a cleanup thread.
pub struct ChannelDrop<T>(Option<Box<T>>)
where
    T: 'static + Send;

impl<T> ChannelDrop<T>
where
    T: 'static + Send,
{
    pub fn new(item: T) -> Self {
        Self(Some(Box::new(item)))
    }
}

impl<T> Default for ChannelDrop<T>
where
    T: 'static + Send + Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Deref for ChannelDrop<T>
where
    T: 'static + Send,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("used after free").deref()
    }
}

impl<T> DerefMut for ChannelDrop<T>
where
    T: 'static + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("used after free").deref_mut()
    }
}

impl<T> Drop for ChannelDrop<T>
where
    T: 'static + Send,
{
    fn drop(&mut self) {
        let inner = self.0.take();
        if let Some(inner) = inner {
            match DROP_CHANNEL.0.lock().try_send(inner) {
                Ok(_) => (),
                Err(TrySendError::Full(_)) => {
                    dbg!("ChannelDrop dispose channel full!");
                    ()
                }
                Err(TrySendError::Disconnected(_)) => {
                    dbg!("ChannelDrop dispose channel disconnected!!");
                    ()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{Receiver, TryRecvError};
    use std::thread;

    #[test]
    fn gets_dispose_channel() {
        let r = DROP_CHANNEL.1.lock();
        assert!(r.is_some());
    }

    #[test]
    fn assert_drops() {
        let x = ChannelDrop::new(2usize);
        let r = DROP_CHANNEL.1.lock();
        assert!(r.is_some());
        let r = r.as_ref().unwrap();
        assert!(r.try_recv().is_err());

        //explicit drop
        std::mem::drop(x);
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //block drop
        {
            let _y = ChannelDrop::new(234.9f32);
            assert!(r.try_recv().is_err());
        }
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //threaded drop
        let child = thread::spawn(move || {
            let _z = ChannelDrop::new("foo");
        });
        assert!(child.join().is_ok());
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());
    }

}
