    pub fn run(self) -> (result: RunResult)
        requires
            self.wf(),
        ensures
            result.running.spec_id() == self.spec_id(),
            !result.running.spec_is_interrupted(),
            result.interrupt_reason == self.spec_interrupt_reason(),
            result.user_tda == self.spec_user_tda(),
            result.running.wf(),
            result.running.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| #![auto] result.running.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.running.spec_drop_safe() == self.spec_drop_safe(),
    {
        proof {
            reveal(ReadyThread::wf);
            reveal(RunningThread::wf);
        }
        let mut state: ThreadState = self.state;
        let interrupt_reason: Option<int> = state.take_interrupt_reason();
        let user_tda: Option<int> = state.get_thread_data_area();
        RunResult {
            running: RunningThread::from_state(state),
            interrupt_reason: interrupt_reason,
            user_tda: user_tda,
        }
    }
