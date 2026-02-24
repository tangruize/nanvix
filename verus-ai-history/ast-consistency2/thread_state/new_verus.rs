    pub fn new(
        id: ThreadIdentifier,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ThreadState)
        ensures
            result@.id == id.spec_value(),
            result@.kernel_stack == kernel_stack,
            result@.user_stack == user_stack,
            result@.has_kernel_stack() == kernel_stack.is_some(),
            result@.has_user_stack() == user_stack.is_some(),
            result@.user_tda == user_tda,
            !result@.is_interrupted(),
            result@.locked_mutex_count == 0,
            result@.drop_safe(),
            result.spec_drop_safe(),
            result.wf(),
    {
        proof { reveal(ThreadState::wf); }
        ThreadState {
            id: id,
            kernel_stack: kernel_stack,
            user_stack: user_stack,
            user_tda: user_tda,
            interrupt_reason: None,
            locked_mutex_count: 0usize,
            locked_mutex_set: Vec::new(),
        }
    }
