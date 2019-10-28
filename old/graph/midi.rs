use super::*;
use crate::binding::ParamBindingGet;
use crate::midi::MidiTrigger;
use crate::ptr::SShrPtr;
use crate::time::TimeSched;

/// A graph leaf node that triggers a midi note.
#[derive(GraphLeaf)]
pub struct MidiNote<Chan, Note, Dur, OnVel, OffVel>
where
    Chan: ParamBindingGet<u8>,
    Note: ParamBindingGet<u8>,
    Dur: ParamBindingGet<TimeResched>,
    OnVel: ParamBindingGet<u8>,
    OffVel: ParamBindingGet<u8>,
{
    trigger: SShrPtr<MidiTrigger>,
    chan: Chan,
    note: Note,
    dur: Dur,
    on_vel: OnVel,
    off_vel: OffVel,
}

impl<Chan, Note, Dur, OnVel, OffVel> MidiNote<Chan, Note, Dur, OnVel, OffVel>
where
    Chan: ParamBindingGet<u8>,
    Note: ParamBindingGet<u8>,
    Dur: ParamBindingGet<TimeResched>,
    OnVel: ParamBindingGet<u8>,
    OffVel: ParamBindingGet<u8>,
{
    /// Construct a new `MidiNote`
    ///
    /// # Arguments
    ///
    /// * `trigger` - the trigger to use to execute the note
    /// * `chan` - the binding for the midi channel
    /// * `note` - the binding for the midi note number
    /// * `dur` - the binding for the note duration
    /// * `on_vel` - the binding for the note on velocity
    /// * `off_vel` - the binding for the note off velocity
    pub fn new(
        trigger: SShrPtr<MidiTrigger>,
        chan: Chan,
        note: Note,
        dur: Dur,
        on_vel: OnVel,
        off_vel: OffVel,
    ) -> Self {
        Self {
            trigger,
            chan,
            note,
            dur,
            on_vel,
            off_vel,
        }
    }
}

impl<Chan, Note, Dur, OnVel, OffVel> GraphLeafExec for MidiNote<Chan, Note, Dur, OnVel, OffVel>
where
    Chan: ParamBindingGet<u8>,
    Note: ParamBindingGet<u8>,
    Dur: ParamBindingGet<TimeResched>,
    OnVel: ParamBindingGet<u8>,
    OffVel: ParamBindingGet<u8>,
{
    fn exec_leaf(&mut self, context: &mut dyn SchedContext) {
        let chan = self.chan.get();
        let note = self.note.get();
        let dur = self.dur.get();
        let on_vel = self.on_vel.get();
        let off_vel = self.off_vel.get();
        self.trigger.lock().note_with_dur(
            TimeSched::ContextRelative(0),
            dur,
            context.as_schedule_trigger_mut(),
            chan,
            note,
            on_vel,
            off_vel,
        );
    }
}
