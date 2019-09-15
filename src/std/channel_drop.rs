use spin::Mutex;
use std::convert::From;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref DROP_BOX_CHANNEL: (
        Mutex<SyncSender<Box<dyn Send>>>,
        Mutex<Option<Receiver<Box<dyn Send>>>>) = {
        let (s, r) = sync_channel(1024);
        (Mutex::new(s), Mutex::new(Some(r)))
    };

    static ref DROP_ARC_CHANNEL: (
        Mutex<SyncSender<Arc<dyn Send + Sync>>>,
        Mutex<Option<Receiver<Arc<dyn Send + Sync>>>>) = {
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
    let rb = DROP_BOX_CHANNEL
        .1
        .lock()
        .take()
        .expect("called consume twice");
    let ra = DROP_ARC_CHANNEL
        .1
        .lock()
        .take()
        .expect("called consume twice");
    Some(Box::new(move || {
        let mut should_loop = true;
        while should_loop {
            should_loop = false;
            match rb.try_recv() {
                Ok(_) => should_loop = true,
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return Err(TryRecvError::Disconnected),
            }
            match ra.try_recv() {
                Ok(_) => should_loop = true,
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return Err(TryRecvError::Disconnected),
            }
        }
        Ok(())
    }))
}

/// A wrapper for a Boxed item that when dropped, will push its boxed item to a channel so that it
/// can be cleaned up in a cleanup thread.
pub struct ChannelDropBox<T>(Option<Box<T>>)
where
    T: 'static + Send;

pub struct ChannelDropArc<T>(Option<Arc<T>>)
where
    T: 'static + Send + Sync;

impl<T> ChannelDropBox<T>
where
    T: 'static + Send,
{
    pub fn new(item: T) -> Self {
        Self(Some(Box::new(item)))
    }
}

impl<T> From<Box<T>> for ChannelDropBox<T>
where
    T: 'static + Send,
{
    fn from(item: Box<T>) -> Self {
        Self(Some(item))
    }
}

impl<T> Default for ChannelDropBox<T>
where
    T: 'static + Send + Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Deref for ChannelDropBox<T>
where
    T: 'static + Send,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("used after free").deref()
    }
}

impl<T> DerefMut for ChannelDropBox<T>
where
    T: 'static + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("used after free").deref_mut()
    }
}

impl<T> Drop for ChannelDropBox<T>
where
    T: 'static + Send,
{
    fn drop(&mut self) {
        let inner = self.0.take();
        if let Some(inner) = inner {
            match DROP_BOX_CHANNEL.0.lock().try_send(inner) {
                Ok(_) => (),
                Err(TrySendError::Full(_)) => {
                    dbg!("ChannelDropBox dispose channel full!");
                    ()
                }
                Err(TrySendError::Disconnected(_)) => {
                    dbg!("ChannelDropBox dispose channel disconnected!!");
                    ()
                }
            }
        }
    }
}

impl<T> ChannelDropArc<T>
where
    T: 'static + Send + Sync,
{
    pub fn new(item: T) -> Self {
        Self(Some(Arc::new(item)))
    }
}

impl<T> From<Arc<T>> for ChannelDropArc<T>
where
    T: 'static + Send + Sync,
{
    fn from(item: Arc<T>) -> Self {
        Self(Some(item))
    }
}

impl<T> Default for ChannelDropArc<T>
where
    T: 'static + Send + Sync + Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Deref for ChannelDropArc<T>
where
    T: 'static + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("used after free").deref()
    }
}

impl<T> Clone for ChannelDropArc<T>
where
    T: 'static + Send + Sync,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Drop for ChannelDropArc<T>
where
    T: 'static + Send + Sync,
{
    fn drop(&mut self) {
        let inner = self.0.take();
        if let Some(inner) = inner {
            if Arc::strong_count(&inner) <= 1 {
                match DROP_ARC_CHANNEL.0.lock().try_send(inner) {
                    Ok(_) => (),
                    Err(TrySendError::Full(_)) => {
                        dbg!("ChannelDropArc dispose channel full!");
                        ()
                    }
                    Err(TrySendError::Disconnected(_)) => {
                        dbg!("ChannelDropArc dispose channel disconnected!!");
                        ()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn gets_dispose_channel() {
        let r = DROP_BOX_CHANNEL.1.lock();
        assert!(r.is_some());

        let r = DROP_ARC_CHANNEL.1.lock();
        assert!(r.is_some());
    }

    #[test]
    fn assert_box_drops() {
        let x = ChannelDropBox::new(2usize);
        let r = DROP_BOX_CHANNEL.1.lock();
        assert!(r.is_some());
        let r = r.as_ref().unwrap();
        assert!(r.try_recv().is_err());

        //explicit drop
        assert_eq!(2usize, *x);
        std::mem::drop(x);
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //block drop
        {
            let mut y = ChannelDropBox::new(234.9f32);
            assert!(r.try_recv().is_err());
            assert_eq!(234.9f32, *y);
            *y = 345.0f32;
            assert_eq!(345.0f32, *y);
        }
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //threaded drop
        let child = thread::spawn(move || {
            let z = ChannelDropBox::new("foo");
            assert_eq!("foo", *z);
        });
        assert!(child.join().is_ok());
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //threaded drop
        let z = ChannelDropBox::new("foo");
        let child = thread::spawn(move || {
            assert_eq!("foo", *z);
            std::mem::drop(z);
        });
        assert!(child.join().is_ok());
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        let b = Box::new(2usize);
        {
            let z: ChannelDropBox<usize> = b.into();
            assert_eq!(2usize, *z);
            assert!(r.try_recv().is_err());
        }
        assert!(r.try_recv().is_ok());
    }

    #[test]
    fn assert_arc_drops() {
        let x = ChannelDropArc::new(2usize);
        let r = DROP_ARC_CHANNEL.1.lock();
        assert!(r.is_some());
        let r = r.as_ref().unwrap();
        assert!(r.try_recv().is_err());

        //explicit drop
        std::mem::drop(x);
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //block drop
        {
            let y = ChannelDropArc::new(234.9f32);
            assert_eq!(234.9f32, *y);
            assert!(r.try_recv().is_err());
        }
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //threaded drop
        let child = thread::spawn(move || {
            let _z = ChannelDropArc::new("foo");
        });
        assert!(child.join().is_ok());
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        let z = ChannelDropArc::new("foo");
        let child = thread::spawn(move || {
            let p = z.clone();
            assert_eq!("foo", *p);
        });
        assert!(child.join().is_ok());
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //clones
        let x = ChannelDropArc::new(2usize);
        assert!(r.try_recv().is_err());
        let y = x.clone();
        assert!(r.try_recv().is_err());
        std::mem::drop(x);
        assert!(r.try_recv().is_err());
        std::mem::drop(y);
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //block drop
        {
            let y = ChannelDropArc::new(42);
            let z = y.clone();
            let p = z.clone();
            assert_eq!(42, *p);
            assert_eq!(42, *z);
            assert!(r.try_recv().is_err());
        }
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //threaded drop
        let child = thread::spawn(move || {
            let y = ChannelDropArc::new("foo");
            let z = y.clone();
            let p = z.clone();
            assert_eq!("foo", *p);
        });
        assert!(child.join().is_ok());
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //threaded drop
        {
            let y = ChannelDropArc::new("foo");
            let z = y.clone();
            let child = thread::spawn(move || {
                let p = z.clone();
                assert_eq!("foo", *p);
            });
            assert!(child.join().is_ok());
            assert!(r.try_recv().is_err());
            assert_eq!("foo", *y);
        }
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());

        //into/from
        let x = Arc::new(99usize);
        {
            let z: ChannelDropArc<usize> = x.into();
            assert_eq!(99usize, *z);
            assert!(r.try_recv().is_err());
        }
        assert!(r.try_recv().is_ok());
        assert!(r.try_recv().is_err());
    }

}
