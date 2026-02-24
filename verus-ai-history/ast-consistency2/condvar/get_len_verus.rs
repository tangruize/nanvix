    pub fn get_len(&self) -> (result: usize)
        requires
            self.wf(),
        ensures
            result as nat == self@.spec_len(),
    {
        self.len
    }
