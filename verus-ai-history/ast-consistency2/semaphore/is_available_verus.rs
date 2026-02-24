    pub fn is_available(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_available(),
    {
        proof { reveal(Semaphore::wf); }
        self.value > 0
    }
