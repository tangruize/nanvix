    pub fn interrupt(self, reason: int) -> (result: InterruptedThread)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            result.spec_id() == self.spec_id(),
            result.spec_reason() == reason,
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
    {
        InterruptedThread::from_state(self.state, reason)
    }
