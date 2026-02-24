    pub fn is_satisfied(&self) -> (result: bool)
        requires
            self.inv(),
        ensures
            result == self@.is_satisfied(),
    {
        self.count >= self.total
    }
