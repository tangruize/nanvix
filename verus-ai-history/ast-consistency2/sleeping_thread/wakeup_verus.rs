    pub fn wakeup(self) -> (result: ReadyThread)
        requires
            self.wf(),
        ensures
            result.spec_id() == self.spec_id(),
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
            result.spec_admission_time() >= 0,
    {
        ReadyThread::from_state(self.state)
    }
