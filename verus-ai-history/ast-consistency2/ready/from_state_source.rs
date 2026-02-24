    pub fn from_state(state: Box<ThreadState>) -> Self {
        Self {
            state,
            admission_time: clock::now(),
        }
    }
