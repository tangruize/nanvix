    pub fn exit(mut self, status: ExitStatus) -> (ZombieThread, *mut ContextInformation) {
        let ctx: *mut ContextInformation = self.state.context_mut();
        (ZombieThread::from_state(self.state, status), ctx)
    }
