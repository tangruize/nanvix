    fn default() -> (result: Capabilities)
        ensures
            result.wf(),
            result@.granted =~= Set::<Capability>::empty(),
    {
        proof {
            reveal(Capabilities::wf);
            reveal(Capabilities::spec_default);
            assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            Capabilities::lemma_default_empty_set();
        }
        Capabilities { bits: 0u8 }
    }
