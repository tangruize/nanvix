    pub(super) fn new(
        id: ThreadIdentifier,
        kernel_stack: Option<KernelStack>,
        user_stack: Option<UserStack>,
        user_tda: Option<VirtualAddress>,
        context: ContextInformation,
        fpu_state: FpuState,
    ) -> Self {
        Self {
            id,
            context: Box::pin(context),
            kernel_stack,
            user_stack,
            user_tda,
            join_cond: Condvar::new(),
            locked_mutexes: BTreeMap::new(),
            interrupt_reason: None,
            fpu_state: Box::pin(fpu_state),
        }
    }
