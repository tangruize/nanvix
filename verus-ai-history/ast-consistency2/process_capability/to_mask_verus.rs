    fn to_mask(capability: Capability) -> (result: u8)
        ensures
            result == Self::spec_mask(capability),
            result == Self::spec_pow2_mask(capability.spec_discriminant()),
    {
        proof {
            Capabilities::lemma_mask_matches_discriminant(capability);
        }
        match capability {
            Capability::ExceptionControl => 1u8,
            Capability::InterruptControl => 2u8,
            Capability::IoManagement => 4u8,
            Capability::MemoryManagement => 8u8,
            Capability::ProcessManagement => 16u8,
        }
    }
