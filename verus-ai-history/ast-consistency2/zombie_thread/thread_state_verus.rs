    pub fn thread_state(&self) -> (result: &ThreadState)
        requires
            self.wf(),
        ensures
            result.spec_id() == self@.spec_id(),
            result@ == self@.state,
    {
        &self.state
    }
