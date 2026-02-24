    pub fn take_kernel_stack(&mut self) -> (result: Option<int>)
        requires
            old(self).wf(),
        ensures
            result == old(self)@.kernel_stack,
            self@.kernel_stack.is_none(),
            !self@.has_kernel_stack(),
            self@.id == old(self)@.id,
            self@.user_stack == old(self)@.user_stack,
            self@.user_tda == old(self)@.user_tda,
            self@.interrupt_reason == old(self)@.interrupt_reason,
            self@.locked_mutex_count == old(self)@.locked_mutex_count,
            forall|a: int| self@.has_mutex(a) == old(self)@.has_mutex(a),
            self.wf(),
    {
        proof { reveal(ThreadState::wf); }
        let stack: Option<int> = self.kernel_stack;
        self.kernel_stack = None;
        stack
    }
