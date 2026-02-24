    pub fn schedule(mut self) -> (ReadyThread, *mut ContextInformation) {
        let ctx: *mut ContextInformation = self.state.context_mut();
        (ReadyThread::from_state(self.state), ctx)
    }
