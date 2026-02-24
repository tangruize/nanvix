    pub fn into_raw_value(self) -> (result: usize)
        requires
            self.inv(),
        ensures result as int == self@.raw_value()
    {
        self.raw_addr
    }
