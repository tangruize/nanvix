    pub fn sleep(mut self, alarm: Option<SystemTime>) -> (SleepingThread, *mut ContextInformation) {
        let ctx: *mut ContextInformation = self.state.context_mut();
        (SleepingThread::from_state(self.state, alarm), ctx)
    }
