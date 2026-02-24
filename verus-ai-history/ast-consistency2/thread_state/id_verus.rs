    pub fn id(&self) -> (result: ThreadIdentifier)
        ensures
            result.spec_value() == self@.id,
    {
        self.id
    }
