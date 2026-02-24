    pub fn schedule(self) -> (result: ReadyThread)
        requires
            self.wf(),
        ensures
            result@.state == self@.state,
            result@.spec_id() == self@.spec_id(),
            result@.spec_locked_mutex_count() == self@.spec_locked_mutex_count(),
            forall|a: int| result@.spec_has_mutex(a) == self@.spec_has_mutex(a),
            result@.spec_drop_safe() == self@.spec_drop_safe(),
            result.wf(),
    {
        proof { reveal(RunningThread::wf); }
        ReadyThread::from_state(self.state)
    }
