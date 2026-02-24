    pub fn new(
        id: ThreadIdentifier,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        ensures
            result.spec_id() == id.spec_value(),
            result.spec_kernel_stack() == kernel_stack,
            result.spec_user_stack() == user_stack,
            result.spec_user_tda() == user_tda,
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            result.spec_drop_safe(),
            result.wf(),
            result.spec_admission_time() >= 0,
    {
        proof { reveal(ReadyThread::wf); }
        let state: ThreadState = ThreadState::new(id, kernel_stack, user_stack, user_tda);
        proof { state.lemma_zero_mutexes_is_drop_safe(); }
        ReadyThread {
            state: state,
            admission_time: clock_now(),
        }
    }
