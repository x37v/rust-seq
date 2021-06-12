use crate::{event::EventEvalContext, tick::TickSched};

pub trait MidiTryEnqueue: Sized {
    fn note_try_enqueue(
        _context: &mut dyn EventEvalContext<Self>,
        _time: TickSched,
        _on: bool,
        _chan: u8,
        _num: u8,
        _vel: u8,
    ) -> Result<(), Self> {
        unimplemented!();
    }
}
