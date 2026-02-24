    pub fn to_ne_bytes(&self) -> (result: [u8; 4])
        requires
            self.inv(),
        ensures
            result == self@.to_ne_bytes_spec(),
    {
        self.value.to_ne_bytes()
    }
