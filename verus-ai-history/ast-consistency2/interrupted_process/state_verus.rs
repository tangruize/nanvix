    pub fn state(&self) -> (result: u64)
        requires
            self.wf(),
        ensures
            result as int == self@.pid,
    {
        self.pid
    }
