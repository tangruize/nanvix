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
        assert(0u8 & 1u8 == 0u8) by (bit_vector);
        assert(0u8 & 2u8 == 0u8) by (bit_vector);
        assert(0u8 & 4u8 == 0u8) by (bit_vector);
        assert(0u8 & 8u8 == 0u8) by (bit_vector);
        assert(0u8 & 16u8 == 0u8) by (bit_vector);
    }

    /// Lemma: The spec_mask for each variant is a single distinct bit.
    pub proof fn lemma_masks_distinct()
        ensures
            Capabilities::spec_mask(Capability::ExceptionControl) == 1u8,
            Capabilities::spec_mask(Capability::InterruptControl) == 2u8,
            Capabilities::spec_mask(Capability::IoManagement) == 4u8,
            Capabilities::spec_mask(Capability::MemoryManagement) == 8u8,
            Capabilities::spec_mask(Capability::ProcessManagement) == 16u8,
    {
    }

    /// Lemma: Setting a capability bit ensures `has` returns true for that capability.
    pub proof fn lemma_set_then_has(pre: Capabilities, cap: Capability)
        ensures
            ({
                let post_bits: u8 = pre.spec_set(cap);
                (post_bits & Self::spec_mask(cap)) != 0u8
            }),
    {
        let mask: u8 = Self::spec_mask(cap);
        let b: u8 = pre.spec_bits();
        let result: u8 = (b | mask) as u8;
        assert((result & mask) != 0u8) by (bit_vector)
            requires
                result == (b | mask) as u8,
        ;
    }

    /// Lemma: Clearing a capability bit ensures `has` returns false for that capability.
    pub proof fn lemma_clear_then_not_has(pre: Capabilities, cap: Capability)
        ensures
            ({
                let post_bits: u8 = pre.spec_clear(cap);
                (post_bits & Self::spec_mask(cap)) == 0u8
            }),
    {
        let mask: u8 = Self::spec_mask(cap);
        let b: u8 = pre.spec_bits();
        let result: u8 = (b & !mask) as u8;
        assert((result & mask) == 0u8) by (bit_vector)
            requires
                result == (b & !mask) as u8,
        ;
    }

    /// Lemma: Distinct capabilities have disjoint (non-overlapping) masks.
    pub proof fn lemma_distinct_masks_disjoint(a: Capability, b: Capability)
        requires
            Self::spec_mask(a) != Self::spec_mask(b),
        ensures
            Self::spec_mask(a) & Self::spec_mask(b) == 0u8,
    {
        assert(1u8 & 2u8 == 0u8) by (bit_vector);
        assert(1u8 & 4u8 == 0u8) by (bit_vector);
        assert(1u8 & 8u8 == 0u8) by (bit_vector);
        assert(1u8 & 16u8 == 0u8) by (bit_vector);
        assert(2u8 & 4u8 == 0u8) by (bit_vector);
        assert(2u8 & 8u8 == 0u8) by (bit_vector);
        assert(2u8 & 16u8 == 0u8) by (bit_vector);
        assert(4u8 & 8u8 == 0u8) by (bit_vector);
        assert(4u8 & 16u8 == 0u8) by (bit_vector);
        assert(8u8 & 16u8 == 0u8) by (bit_vector);
    }

    /// Lemma: Setting a capability preserves other bits.
    pub proof fn lemma_set_preserves_other(pre: Capabilities, cap_set: Capability, cap_other: Capability)
        requires
            Self::spec_mask(cap_set) != Self::spec_mask(cap_other),
        ensures
            ({
                let post_bits: u8 = pre.spec_set(cap_set);
                ((post_bits & Self::spec_mask(cap_other)) != 0u8)
                ==
                ((pre.spec_bits() & Self::spec_mask(cap_other)) != 0u8)
            }),
    {
        let mask_s: u8 = Self::spec_mask(cap_set);
        let mask_o: u8 = Self::spec_mask(cap_other);
        let b: u8 = pre.spec_bits();

        Self::lemma_distinct_masks_disjoint(cap_set, cap_other);

        assert(((b | mask_s) as u8 & mask_o != 0u8) == (b & mask_o != 0u8)) by (bit_vector)
            requires
                mask_s & mask_o == 0u8,
        ;
    }

    /// Lemma: Clearing a capability preserves other bits.
    pub proof fn lemma_clear_preserves_other(pre: Capabilities, cap_clear: Capability, cap_other: Capability)
        requires
            Self::spec_mask(cap_clear) != Self::spec_mask(cap_other),
        ensures
            ({
                let post_bits: u8 = pre.spec_clear(cap_clear);
                ((post_bits & Self::spec_mask(cap_other)) != 0u8)
                ==
                ((pre.spec_bits() & Self::spec_mask(cap_other)) != 0u8)
            }),
    {
        let mask_c: u8 = Self::spec_mask(cap_clear);
        let mask_o: u8 = Self::spec_mask(cap_other);
        let b: u8 = pre.spec_bits();

        Self::lemma_distinct_masks_disjoint(cap_clear, cap_other);

        assert(((b & !mask_c) as u8 & mask_o != 0u8) == (b & mask_o != 0u8)) by (bit_vector)
            requires
                mask_c & mask_o == 0u8,
        ;
    }

    /// Lemma: Setting an already-set bit is idempotent.
    pub proof fn lemma_set_idempotent(pre: Capabilities, cap: Capability)
        requires
            pre.spec_has(cap),
        ensures
            pre.spec_set(cap) == pre.spec_bits(),
    {
        let mask: u8 = Self::spec_mask(cap);
        let b: u8 = pre.spec_bits();
        assert((b | mask) as u8 == b) by (bit_vector)
            requires
                (b & mask) != 0u8,
                mask == 1u8 || mask == 2u8 || mask == 4u8 || mask == 8u8 || mask == 16u8,
        ;
    }

    /// Lemma: Clearing an already-clear bit is idempotent.
    pub proof fn lemma_clear_idempotent(pre: Capabilities, cap: Capability)
        requires
            !pre.spec_has(cap),
        ensures
            pre.spec_clear(cap) == pre.spec_bits(),
    {
        let mask: u8 = Self::spec_mask(cap);
        let b: u8 = pre.spec_bits();
        assert((b & !mask) as u8 == b) by (bit_vector)
            requires
                (b & mask) == 0u8,
                mask == 1u8 || mask == 2u8 || mask == 4u8 || mask == 8u8 || mask == 16u8,
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
