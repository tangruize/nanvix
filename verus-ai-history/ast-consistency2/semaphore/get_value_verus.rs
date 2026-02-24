    pub fn get_value(&self) -> (result: usize)
        requires
            self.wf(),
        ensures
            result as nat == self@.value,
    {
        proof { reveal(Semaphore::wf); }
        self.value
    }
