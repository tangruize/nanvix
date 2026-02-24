    pub fn into_isize(self) -> (result: isize)
        requires
            self.inv(),
        ensures
            result as int == self@.value,
    {
        self.value as isize
    }
