# rust-sched
a work in progress scheduled executor, built in rust

The info below is likely outdated.

## Terminology

### Scheduler

The object that allows initial scheduling of objects that implement the
SchedCall trait. This object then passes these off to an Executor which
executes the schedule.

### Executor

The object that a schedule gets executed in. This will usually be in another
thread and likely in a context where heap allocation should not be allowed.

### Context

The object that the scheduled object can use to discover features, for
instance, the current time, or, in a musical bar/beat scenario, how to
translate a bar/beat into a tick value.

### Cache

A source for getting objects to schedule in the Executor context, avoiding heap
allocation. For instance, a MIDI on/off note pair to be scheduled.
	
### Sink

An object that accepts objects to send out from the Executor context. For
instance, the actual MIDI data to go to your hardware.


## NOTES

sched should work like channel, you get a sender and receiver, the receiver goes into another thread that can execute.
the sender should impl an interface that is the same [or most of] the interface that the receiver thread provides to
its functions for additional scheduling.

we can provide a variety of different contexts that you can push the receiver to and should be able to send multiple receivers there..
like, jack.

the executor would have a send and receive as well so you can add more scheds.
impl JackExecutorSender {
	fn add(receiver to execute);
}

