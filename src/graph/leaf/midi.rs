use crate::{
    event::{midi::MidiTryEnqueue, EventEvalContext},
    graph::{ChildCount, GraphChildExec, GraphNodeExec},
    param::ParamGet,
    tick::{TickResched, TickSched},
};

pub struct MidiNote<N, C, D, VN, VF> {
    note: N,
    chan: C,
    dur: D,
    vel_on: VN,
    vel_off: VF,
}

impl<N, C, D, VN, VF> MidiNote<N, C, D, VN, VF>
where
    N: ParamGet<u8>,
    C: ParamGet<u8>,
    D: ParamGet<TickResched>,
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

impl<N, C, D, VN, VF, E> GraphNodeExec<E> for MidiNote<N, C, D, VN, VF>
where
    N: ParamGet<u8>,
    C: ParamGet<u8>,
    D: ParamGet<TickResched>,
    VN: ParamGet<u8>,
    VF: ParamGet<u8>,
    E: MidiTryEnqueue,
{
    fn graph_exec(&self, context: &mut dyn EventEvalContext<E>, _children: &dyn GraphChildExec<E>) {
        let on = TickSched::ContextRelative(0);
        let off = on.add(self.dur.get(), context.as_tick_context());
        let num = num_traits::clamp(self.note.get(), 0, 127);
        let chan = num_traits::clamp(self.chan.get(), 0, 15);
        let vel = num_traits::clamp(self.vel_off.get(), 0, 127);
        //schedule off first so we don't have a stuck note
        let off = E::note_try_enqueue(context, off, false, chan, num, vel);
        if off.is_ok() {
            let vel = num_traits::clamp(self.vel_on.get(), 1, 127);
            let _on = E::note_try_enqueue(context, on, true, chan, num, vel);
        }
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::None
    }
}
