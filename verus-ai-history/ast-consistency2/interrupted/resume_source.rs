    pub fn resume(mut self) -> ReadyThread {
        self.state.set_interrupt_reason(self.reason);
        ReadyThread::from_state(self.state)
    }
