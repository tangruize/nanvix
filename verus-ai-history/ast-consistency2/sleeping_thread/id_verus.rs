    pub fn id(&self) -> (result: ThreadIdentifier)
        requires
            self.wf(),
        ensures
            result.spec_value() == self.spec_id(),
    {
        self.state.id()
    }
