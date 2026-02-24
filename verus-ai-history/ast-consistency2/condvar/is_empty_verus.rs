    pub fn is_empty(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.spec_is_empty(),
    {
        self.len == 0
    }
