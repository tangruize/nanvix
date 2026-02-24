    pub fn resume(self) -> (result: ReadyThread)
        requires
            self.wf(),
        ensures
            result@.spec_id() == self@.spec_id(),
            result@.spec_interrupt_reason() == Some(self@.spec_reason()),
            result.wf(),
    {
        proof { reveal(InterruptedThread::wf); }
        let reason_value: int = self.reason;
        let mut state: ThreadState = self.state;
        state.set_interrupt_reason(reason_value);
        ReadyThread::from_state(state)
    }
