    pub fn admission_time(&self) -> (result: int)
        ensures
            result == self.spec_admission_time(),
    {
        self.admission_time
    }
