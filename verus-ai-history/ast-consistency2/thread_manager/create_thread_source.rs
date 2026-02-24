    pub fn create_thread(
        &mut self,
        kernel_stack: Option<KernelStack>,
        user_stack: Option<UserStack>,
        user_tda: Option<VirtualAddress>,
        context: ContextInformation,
    ) -> ReadyThread {
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from(<i32>::from(self.next_id) + 1);

        ReadyThread::new(
            id,
            kernel_stack,
            user_stack,
            user_tda,
            context,
            // SAFETY: calls to FpuState::new are synchronized.
            unsafe { FpuState::new() },
        )
    }
