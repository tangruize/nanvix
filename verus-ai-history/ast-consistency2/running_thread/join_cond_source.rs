    pub fn join_cond(&self) -> Condvar {
        // NOTE: we must wake up all, otherwise some threads can be left waiting forever.
        self.state.join_cond()
    }
