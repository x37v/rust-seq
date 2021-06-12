use crate::{
    event::{midi::MidiTryEnqueue, EventEvalContext},
    graph::GraphLeafExec,
    param::ParamGet,
    tick::TickSched,
};

pub struct Note<N, C, D, VN, VF> {
    note: N,
    chan: C,
    dur: D,
    vel_on: VN,
    vel_off: VF,
}

impl<N, C, D, VN, VF> Note<N, C, D, VN, VF>
where
    N: ParamGet<u8>,
    C: ParamGet<u8>,
    D: ParamGet<TickSched>,
    VN: ParamGet<u8>,
    VF: ParamGet<u8>,
{
    pub fn new(note: N, chan: C, dur: D, vel_on: VN, vel_off: VF) -> Self {
        Self {
            note,
            chan,
            dur,
            vel_on,
            vel_off,
        }
    }
}

impl<N, C, D, VN, VF, E> GraphLeafExec<E> for Note<N, C, D, VN, VF>
where
    N: ParamGet<u8>,
    C: ParamGet<u8>,
    D: ParamGet<TickSched>,
    VN: ParamGet<u8>,
    VF: ParamGet<u8>,
    E: Send + MidiTryEnqueue,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>) {
        let num = self.note.get();
        let chan = self.chan.get();
        let dur = self.dur.get();
        //schedule off first so we don't have a stuck note
        let off = E::note_try_enqueue(context, dur, false, chan, num, self.vel_off.get());
        if off.is_ok() {
            let _on = E::note_try_enqueue(
                context,
                TickSched::ContextRelative(0),
                true,
                chan,
                num,
                self.vel_on.get(),
            );
        }
    }
}
