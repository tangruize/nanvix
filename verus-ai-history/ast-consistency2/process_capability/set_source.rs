    pub fn set(&mut self, capability: Capability) {
        self.0 |= 1 << capability as u8;
    }
