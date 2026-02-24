    pub fn pid(&self) -> (result: ProcessIdentifier)
        ensures
            result.spec_value() == self.spec_pid(),
    {
        self.pid
    }
