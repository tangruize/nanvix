    pub fn initial_sp(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            result as int > self.spec_base(),
    {
        self.top_raw()
    }
