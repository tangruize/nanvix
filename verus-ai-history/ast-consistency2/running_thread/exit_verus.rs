    pub fn exit(self, status: int) -> (result: ZombieThread)
        requires
            self.wf(),
        ensures
            result@.state == self@.state,
            result@.spec_id() == self@.spec_id(),
            result@.spec_status() == status,
            result@.spec_locked_mutex_count() == self@.spec_locked_mutex_count(),
            forall|a: int| result@.spec_has_mutex(a) == self@.spec_has_mutex(a),
            result@.spec_drop_safe() == self@.spec_drop_safe(),
            result.wf(),
    {
        proof { reveal(RunningThread::wf); }
        ZombieThread::from_state(self.state, status)
    }
