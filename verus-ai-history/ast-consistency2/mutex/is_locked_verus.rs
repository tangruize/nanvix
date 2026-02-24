    pub fn is_locked(&self) -> (result: bool)
        ensures
            result == self@.locked,
            result == self@.is_locked(),
    {
        self.locked
    }
