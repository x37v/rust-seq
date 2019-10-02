use crate::item_sink::{ItemDispose, ItemDisposeFunc, ItemSink};
use std::ops::DerefMut;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};

pub struct ChannelItemSink<T>
where
    T: Send,
{
    send: SyncSender<T>,
}

pub struct ChannelItemDispose<T>
where
    T: Send,
{
    recv: Receiver<T>,
}

impl<T> ItemSink<T> for ChannelItemSink<T>
where
    T: Send,
{
    fn try_put(&mut self, item: T) -> Result<(), T> {
        match self.send.try_send(item) {
            Ok(()) => Ok(()),
            Err(TrySendError::Disconnected(item)) => Err(item),
            Err(TrySendError::Full(item)) => Err(item),
        }
    }
}

impl<T> ItemDispose<T> for ChannelItemDispose<T>
where
    T: Send,
{
    fn dispose_all(&mut self) -> Result<(), ()> {
        //do nothing, just let go
        self.with_each(&|_| {})
    }
}

impl<T> ItemDisposeFunc<T> for ChannelItemDispose<T>
where
    T: Send,
{
    fn with_each(&mut self, func: &dyn Fn(T)) -> Result<(), ()> {
        let mut should_loop = true;
        while should_loop {
            should_loop = false;
            match self.recv.try_recv() {
                Ok(v) => {
                    should_loop = true;
                    func(v)
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return Err(()),
            }
        }
        Ok(())
    }
}
