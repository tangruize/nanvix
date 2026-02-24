    pub fn new(
        id: ThreadIdentifier,
        kernel_stack: Option<KernelStack>,
        user_stack: Option<UserStack>,
        user_tda: Option<VirtualAddress>,
        context: ContextInformation,
        fpu_state: FpuState,
    ) -> Self {
        Self {
            state: Box::new(ThreadState::new(
                id,
                kernel_stack,
                user_stack,
                user_tda,
                context,
                fpu_state,
            )),
            admission_time: clock::now(),
        }
    }
