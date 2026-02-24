    pub fn interrupt(self, reason: InterruptReason) -> InterruptedThread {
        InterruptedThread::from_state(self.state, reason)
    }
