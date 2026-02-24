    pub fn timer_handler_model(&mut self)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            self@.ticks == old(self)@.next_ticks(),
            old(self)@.is_max() ==> self@.ticks == 0,
            !old(self)@.is_max() ==> self@.ticks == old(self)@.ticks + 1,
    {
        self.increment();
    }
