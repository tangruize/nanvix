    pub fn has_capability(&self, capability: Capability) -> bool {
        self.capabilities.has(capability)
    }
