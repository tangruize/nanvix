    pub fn get_count(&self) -> (result: usize)
        ensures
            result as nat == self@.count,
    {
        self.count
    }
