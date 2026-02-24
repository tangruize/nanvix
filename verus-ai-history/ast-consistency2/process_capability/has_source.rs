    pub fn has(&self, capability: Capability) -> bool {
        (self.0 & (1 << capability as u8)) != 0
    }
