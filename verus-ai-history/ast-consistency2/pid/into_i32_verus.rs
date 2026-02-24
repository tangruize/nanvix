    pub fn into_i32(self) -> (result: i32)
        requires
            self.inv(),
        ensures
            result as int == self@.value,
    {
        self.value
    }
