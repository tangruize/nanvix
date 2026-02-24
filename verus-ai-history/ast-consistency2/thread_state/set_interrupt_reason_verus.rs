    pub fn set_interrupt_reason(&mut self, reason: int)
        requires
            old(self).wf(),
        ensures
            self@.is_interrupted(),
            self@.interrupt_reason == Some(reason),
            self@.id == old(self)@.id,
            self@.kernel_stack == old(self)@.kernel_stack,
            self@.user_stack == old(self)@.user_stack,
            self@.user_tda == old(self)@.user_tda,
            self@.locked_mutex_count == old(self)@.locked_mutex_count,
            forall|a: int| self@.has_mutex(a) == old(self)@.has_mutex(a),
            self.wf(),
    {
        proof { reveal(ThreadState::wf); }
        self.interrupt_reason = Some(reason);
    }
