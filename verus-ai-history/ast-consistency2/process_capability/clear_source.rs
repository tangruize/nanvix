    pub fn clear(&mut self, capability: Capability) {
        self.0 &= !(1 << capability as u8);
    }
