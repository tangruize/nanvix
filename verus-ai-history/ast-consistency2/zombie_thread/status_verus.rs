    pub fn status(&self) -> (result: int)
        requires
            self.wf(),
        ensures
            result == self@.spec_status(),
    {
        self.status
    }
