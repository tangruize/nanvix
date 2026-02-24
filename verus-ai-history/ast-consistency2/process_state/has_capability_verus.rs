    pub fn has_capability(&self, capability: Capability) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.capabilities_granted.contains(capability),
    {
        proof {
            self.capabilities.lemma_view_bits();
        }
        self.capabilities.has(capability)
    }
