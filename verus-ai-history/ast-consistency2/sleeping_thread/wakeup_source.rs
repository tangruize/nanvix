    pub fn wakeup(self) -> ReadyThread {
        ReadyThread::from_state(self.state)
    }
