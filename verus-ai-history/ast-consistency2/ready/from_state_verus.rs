    pub fn from_state(state: ThreadState) -> (result: ReadyThread)
        requires
            state.wf(),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_interrupt_reason() == state.spec_interrupt_reason(),
            result.spec_user_tda() == state.spec_user_tda(),
            result.spec_kernel_stack() == state.spec_kernel_stack(),
            result.spec_user_stack() == state.spec_user_stack(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| #![auto] result.spec_has_mutex(a) == state@.has_mutex(a),
            result.spec_drop_safe() == state@.drop_safe(),
            result.wf(),
            result.spec_admission_time() >= 0,
    {
        proof { reveal(ReadyThread::wf); }
        ReadyThread {
            state: state,
            admission_time: clock_now(),
        }
    }
