trait Context {
    fn base_tick(&self) -> usize;
    fn context_tick(&self) -> usize;
    fn base_tick_period_micros(&self) -> f32;
    fn context_tick_period_micros(&self) -> f32;
    /*
    fn schedule(&mut self, t: TimeSched, func: SchedFn<SrcSnk, Context>);
    */
}

trait GraphExec {
    fn exec(&mut self, context: &mut impl Context) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
