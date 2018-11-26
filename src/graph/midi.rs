use super::*;
use binding::BindingGetP;
use midi::MidiTrigger;
use TimeSched;

pub struct MidiNote {
    trigger: Arc<spinlock::Mutex<MidiTrigger>>,
    chan: BindingGetP<u8>,
    note: BindingGetP<u8>,
    dur: BindingGetP<TimeResched>,
    on_vel: BindingGetP<u8>,
    off_vel: BindingGetP<u8>,
}

impl MidiNote {
    pub fn new_p(
        trigger: Arc<spinlock::Mutex<MidiTrigger>>,
        chan: BindingGetP<u8>,
        note: BindingGetP<u8>,
        dur: BindingGetP<TimeResched>,
        on_vel: BindingGetP<u8>,
        off_vel: BindingGetP<u8>,
    ) -> Box<Self> {
        Box::new(Self {
            trigger,
            chan,
            note,
            dur,
            on_vel,
            off_vel,
        })
    }
}

impl GraphExec for MidiNote {
    fn exec(&mut self, context: &mut dyn SchedContext, _children: &mut dyn ChildExec) -> bool {
        let chan = self.chan.get();
        let note = self.note.get();
        let dur = self.dur.get();
        let on_vel = self.on_vel.get();
        let off_vel = self.off_vel.get();
        self.trigger.lock().note_with_dur(
            TimeSched::Relative(0),
            dur,
            context.as_schedule_trigger_mut(),
            chan,
            note,
            on_vel,
            off_vel,
        );
        true
    }

    fn children_max(&self) -> ChildCount {
        ChildCount::None
    }
}
