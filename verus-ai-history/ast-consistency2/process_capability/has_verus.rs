    pub fn has(&self, capability: Capability) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.granted.contains(capability),
    {
        proof {
            self.lemma_has_iff_set_contains(capability);
        }
        let mask: u8 = Self::to_mask(capability);
        (self.bits & mask) != 0u8
    }
