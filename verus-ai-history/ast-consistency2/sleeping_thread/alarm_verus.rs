    pub fn alarm(&self) -> (result: Option<int>)
        requires
            self.wf(),
        ensures
            result == self.spec_alarm(),
    {
        self.alarm
    }
