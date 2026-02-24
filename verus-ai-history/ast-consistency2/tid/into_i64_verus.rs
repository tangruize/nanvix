    pub fn into_i64(self) -> (result: i64)
        requires
            self.inv(),
        ensures
            result as int == self@.value,
    {
        self.value as i64
    }
