    pub fn check_drop_safe(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.drop_safe(),
            result == (self@.locked_mutex_count == 0),
    {
        proof { reveal(ThreadState::wf); }
        self.locked_mutex_count == 0
    }
