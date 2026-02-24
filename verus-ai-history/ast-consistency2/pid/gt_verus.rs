    pub fn gt(&self, other: &ProcessIdentifier) -> (result: bool)
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value > other@.value),
    {
        self.value > other.value
    }
