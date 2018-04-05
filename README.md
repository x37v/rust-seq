# rust-seq
a work in progress scheduled executor, built in rust

## NOTES

seq should work like channel, you get a sender and receiver, the receiver goes into another thread that can execute.
the sender should impl an interface that is the same [or most of] the interface that the receiver thread provides to
its functions for additional scheduling.

we can provide a variety of different contexts that you can push the receiver to and should be able to send multiple receivers there..
like, jack.

the executor would have a send and receive as well so you can add more seqs.
impl JackExecutorSender {
	fn add(receiver to execute);
}

