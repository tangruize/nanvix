// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capabilities Proofs.
// This file contains proof lemmas for the Capabilities type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl Capabilities {
    /// Lemma: Default capabilities have no bits set.
    pub proof fn lemma_default_is_empty()
        ensures
            Capabilities::spec_default().spec_bits() == 0u8,
            !Capabilities::spec_default().spec_has(Capability::ExceptionControl),
            !Capabilities::spec_default().spec_has(Capability::InterruptControl),
            !Capabilities::spec_default().spec_has(Capability::IoManagement),
            !Capabilities::spec_default().spec_has(Capability::MemoryManagement),
            !Capabilities::spec_default().spec_has(Capability::ProcessManagement),
    {
    }

    /// Lemma: Setting a capability bit ensures `has` returns true for that capability.
    pub proof fn lemma_set_then_has(pre: Capabilities, cap: Capability)
        ensures
            ({
                let post_bits: u8 = pre.spec_set(cap);
                (post_bits & (1u8 << cap.spec_discriminant())) != 0u8
            }),
    {
        cap.lemma_discriminant_bounds();
        let d: int = cap.spec_discriminant();
        assert(0 <= d <= 4);
        let mask: u8 = (1u8 << d);
        let result: u8 = (pre.spec_bits() | mask) as u8;
        assert(result & mask == mask) by (bit_vector)
            requires
                mask == (1u8 << d),
                result == (pre.spec_bits() | mask) as u8,
                0 <= d <= 4,
        ;
    }

    /// Lemma: Clearing a capability bit ensures `has` returns false for that capability.
    pub proof fn lemma_clear_then_not_has(pre: Capabilities, cap: Capability)
        ensures
            ({
                let post_bits: u8 = pre.spec_clear(cap);
                (post_bits & (1u8 << cap.spec_discriminant())) == 0u8
            }),
    {
        cap.lemma_discriminant_bounds();
        let d: int = cap.spec_discriminant();
        assert(0 <= d <= 4);
        let mask: u8 = (1u8 << d);
        let result: u8 = (pre.spec_bits() & !mask) as u8;
        assert(result & mask == 0u8) by (bit_vector)
            requires
                mask == (1u8 << d),
                result == (pre.spec_bits() & !mask) as u8,
                0 <= d <= 4,
        ;
    }

    /// Lemma: Setting a capability preserves other bits.
    pub proof fn lemma_set_preserves_other(pre: Capabilities, cap_set: Capability, cap_other: Capability)
        requires
            cap_set.spec_discriminant() != cap_other.spec_discriminant(),
        ensures
            ({
                let post_bits: u8 = pre.spec_set(cap_set);
                ((post_bits & (1u8 << cap_other.spec_discriminant())) != 0u8)
                ==
                ((pre.spec_bits() & (1u8 << cap_other.spec_discriminant())) != 0u8)
            }),
    {
        cap_set.lemma_discriminant_bounds();
        cap_other.lemma_discriminant_bounds();
        let ds: int = cap_set.spec_discriminant();
        let do_: int = cap_other.spec_discriminant();
        let mask_s: u8 = (1u8 << ds);
        let mask_o: u8 = (1u8 << do_);
        let result: u8 = (pre.spec_bits() | mask_s) as u8;
        assert((result & mask_o != 0u8) == (pre.spec_bits() & mask_o != 0u8)) by (bit_vector)
            requires
                mask_s == (1u8 << ds),
                mask_o == (1u8 << do_),
                result == (pre.spec_bits() | mask_s) as u8,
                0 <= ds <= 4,
                0 <= do_ <= 4,
                ds != do_,
        ;
    }

    /// Lemma: Clearing a capability preserves other bits.
    pub proof fn lemma_clear_preserves_other(pre: Capabilities, cap_clear: Capability, cap_other: Capability)
        requires
            cap_clear.spec_discriminant() != cap_other.spec_discriminant(),
        ensures
            ({
                let post_bits: u8 = pre.spec_clear(cap_clear);
                ((post_bits & (1u8 << cap_other.spec_discriminant())) != 0u8)
                ==
                ((pre.spec_bits() & (1u8 << cap_other.spec_discriminant())) != 0u8)
            }),
    {
        cap_clear.lemma_discriminant_bounds();
        cap_other.lemma_discriminant_bounds();
        let dc: int = cap_clear.spec_discriminant();
        let do_: int = cap_other.spec_discriminant();
        let mask_c: u8 = (1u8 << dc);
        let mask_o: u8 = (1u8 << do_);
        let result: u8 = (pre.spec_bits() & !mask_c) as u8;
        assert((result & mask_o != 0u8) == (pre.spec_bits() & mask_o != 0u8)) by (bit_vector)
            requires
                mask_c == (1u8 << dc),
                mask_o == (1u8 << do_),
                result == (pre.spec_bits() & !mask_c) as u8,
                0 <= dc <= 4,
                0 <= do_ <= 4,
                dc != do_,
        ;
    }

    /// Lemma: Setting an already-set bit is idempotent.
    pub proof fn lemma_set_idempotent(pre: Capabilities, cap: Capability)
        requires
            pre.spec_has(cap),
        ensures
            pre.spec_set(cap) == pre.spec_bits(),
    {
        cap.lemma_discriminant_bounds();
        let d: int = cap.spec_discriminant();
        let mask: u8 = (1u8 << d);
        assert((pre.spec_bits() | mask) as u8 == pre.spec_bits()) by (bit_vector)
            requires
                (pre.spec_bits() & mask) != 0u8,
                mask == (1u8 << d),
                0 <= d <= 4,
        ;
    }

    /// Lemma: Clearing an already-clear bit is idempotent.
    pub proof fn lemma_clear_idempotent(pre: Capabilities, cap: Capability)
        requires
            !pre.spec_has(cap),
        ensures
            pre.spec_clear(cap) == pre.spec_bits(),
    {
        cap.lemma_discriminant_bounds();
        let d: int = cap.spec_discriminant();
        let mask: u8 = (1u8 << d);
        assert((pre.spec_bits() & !mask) as u8 == pre.spec_bits()) by (bit_vector)
            requires
                (pre.spec_bits() & mask) == 0u8,
                mask == (1u8 << d),
                0 <= d <= 4,
        ;
    }

    /// Lemma: View equality implies bitfield equality.
    pub proof fn lemma_view_equality(a: &Capabilities, b: &Capabilities)
        requires
            a@ == b@,
        ensures
            a.spec_bits() == b.spec_bits(),
    {
    }

    /// Lemma: All capabilities are well-formed.
    pub proof fn lemma_wf(&self)
        ensures
            self.wf(),
    {
    }
}

} // verus!
