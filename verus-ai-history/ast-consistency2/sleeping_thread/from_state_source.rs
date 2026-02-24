    pub(super) fn from_state(state: Box<ThreadState>, alarm: Option<SystemTime>) -> Self {
        Self { state, alarm }
    }
