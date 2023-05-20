use crate::{
    event::{midi::MidiTryEnqueue, EventEvalContext},
    graph::{ChildCount, GraphChildExec, GraphNodeExec},
    param::ParamGet,
    tick::{TickResched, TickSched},
};

pub struct MidiNote<N, C, D, VN, VF, U> {
    note: N,
    chan: C,
    dur: D,
    vel_on: VN,
    vel_off: VF,
    _phantom: core::marker::PhantomData<U>,
}

impl<N, C, D, VN, VF, U> MidiNote<N, C, D, VN, VF, U>
where
    N: ParamGet<u8, U>,
    C: ParamGet<u8, U>,
    D: ParamGet<TickResched, U>,
    VN: ParamGet<u8, U>,
    VF: ParamGet<u8, U>,
{
    pub fn new(note: N, chan: C, dur: D, vel_on: VN, vel_off: VF) -> Self {
        Self {
            note,
            chan,
            dur,
            vel_on,
            vel_off,
            _phantom: Default::default(),
        }
    }
}

impl<N, C, D, VN, VF, E, U> GraphNodeExec<E, U> for MidiNote<N, C, D, VN, VF, U>
where
    N: ParamGet<u8, U>,
    C: ParamGet<u8, U>,
    D: ParamGet<TickResched, U>,
    VN: ParamGet<u8, U>,
    VF: ParamGet<u8, U>,
    E: MidiTryEnqueue,
{
    fn graph_exec(
        &self,
        context: &mut dyn EventEvalContext<E>,
        _children: &dyn GraphChildExec<E, U>,
        user_data: &mut U,
    ) {
        let on = TickSched::ContextRelative(0);
        let off = on.add(self.dur.get(user_data), context.as_tick_context());
        let num = num_traits::clamp(self.note.get(user_data), 0, 127);
        let chan = num_traits::clamp(self.chan.get(user_data), 0, 15);
        let vel = num_traits::clamp(self.vel_off.get(user_data), 0, 127);
        //schedule off first so we don't have a stuck note
        let off = E::note_try_enqueue(context, off, false, chan, num, vel);
        if off.is_ok() {
            let vel = num_traits::clamp(self.vel_on.get(user_data), 1, 127);
            let _on = E::note_try_enqueue(context, on, true, chan, num, vel);
        }
    }
    fn graph_children_max(&self) -> ChildCount {
        ChildCount::None
    }
}
