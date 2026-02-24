    pub fn state(&self) -> (result: u64)
        requires
            self.inv(),
        ensures
            result as int == self@.pid,
    {
        unimplemented!()
    }
