    pub(super) fn from_state(state: Box<ThreadState>, reason: InterruptReason) -> Self {
        Self { state, reason }
    }
