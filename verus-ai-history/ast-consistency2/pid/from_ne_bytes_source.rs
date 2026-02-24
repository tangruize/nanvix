    pub fn from_ne_bytes(bytes: [u8; core::mem::size_of::<i32>()]) -> Self {
        Self(i32::from_ne_bytes(bytes))
    }
