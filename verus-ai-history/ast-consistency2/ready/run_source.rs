    pub fn run(
        mut self,
    ) -> (RunningThread, Option<InterruptReason>, *mut ContextInformation, Option<VirtualAddress>)
    {
        let ctx: *mut ContextInformation = self.state.context_mut();
        let interrupt_reason: Option<InterruptReason> = self.state.take_interrupt_reason();
        let user_tda: Option<VirtualAddress> = self.state.get_thread_data_area();
        (RunningThread::from_state(self.state), interrupt_reason, ctx, user_tda)
    }
