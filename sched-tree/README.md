* tree nodes have an id
    * create a wrapper that can add an id to any schedulable
* tree nodes have a position to schedule them in their parent, so you can have more than one child?
    * to add an item to a schedule list, just schedule it with the parent as the schedule list and then the time is the time into the list [ticks] ... like schedule(parent, Some(234), item) puts that item
* to make something happen "now" just schedule it at the root with 0 as its position and that should happen before any other child
