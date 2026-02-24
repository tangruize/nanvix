    pub fn to_ne_bytes(&self) -> [u8; core::mem::size_of::<i32>()] {
        self.0.to_ne_bytes()
    }
