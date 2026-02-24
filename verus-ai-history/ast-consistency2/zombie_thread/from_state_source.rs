    pub(super) fn from_state(state: Box<ThreadState>, status: ExitStatus) -> Self {
        Self { status, state }
    }
