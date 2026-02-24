    pub fn from_state(state: ThreadState) -> (result: RunningThread)
        requires
            state.wf(),
        ensures
            result@.state == state@,
            result@.spec_id() == state@.id,
            result@.spec_is_interrupted() == state@.is_interrupted(),
            result@.spec_locked_mutex_count() == state@.locked_mutex_count,
            forall|a: int| result@.spec_has_mutex(a) == state@.has_mutex(a),
            result@.spec_drop_safe() == state@.drop_safe(),
            result.wf(),
    {
        proof { reveal(RunningThread::wf); }
        RunningThread { state: state }
    }
