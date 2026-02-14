// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capability Proofs (libs).
// This file contains proof lemmas for the Capability type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl Capability {
    /// Lemma: All capability variants satisfy the invariant.
    pub proof fn lemma_all_variants_inv()
        ensures
            Capability::ExceptionControl.inv(),
            Capability::InterruptControl.inv(),
            Capability::IoManagement.inv(),
            Capability::MemoryManagement.inv(),
            Capability::ProcessManagement.inv(),
    {
    }

    /// Lemma: Each variant has a unique discriminant (via view).
    pub proof fn lemma_discriminants_unique()
        ensures
            Capability::ExceptionControl@.value == 0,
            Capability::InterruptControl@.value == 1,
            Capability::IoManagement@.value == 2,
            Capability::MemoryManagement@.value == 3,
            Capability::ProcessManagement@.value == 4,
    {
    }

    /// Lemma: Any Capability instance satisfies the invariant.
    pub proof fn lemma_inv(&self)
        ensures
            self.inv(),
    {
    }

    /// Lemma: Discriminants are mutually exclusive.
    pub proof fn lemma_discriminants_disjoint(a: &Capability, b: &Capability)
        requires
            a@.value == b@.value,
        ensures
            *a == *b,
    {
    }

    /// Lemma: Round-trip for valid discriminants.
    pub proof fn lemma_try_from_roundtrip(v: u32)
        requires
            CapabilityView::is_valid_discriminant(v as int),
        ensures
            CapabilityView::from_discriminant(v as int).spec_discriminant() == v as int,
    {
    }

    /// Lemma: from_discriminant is a left inverse of spec_discriminant.
    pub proof fn lemma_from_discriminant_inverse(&self)
        ensures
            CapabilityView::from_discriminant(self@.value) == *self,
    {
    }

    /// Lemma: View equality implies variant equality.
    pub proof fn lemma_view_equality(a: &Capability, b: &Capability)
        requires
            a@ == b@,
        ensures
            *a == *b,
    {
    }

    /// Lemma: The discriminant is bounded within [0, 4].
    pub proof fn lemma_discriminant_bounds(&self)
        ensures
            0 <= self@.value <= 4,
    {
    }

    /// Lemma: Total coverage — exactly 5 valid discriminants exist.
    pub proof fn lemma_valid_discriminant_count()
        ensures
            forall|v: int| CapabilityView::is_valid_discriminant(v) <==> (0 <= v <= 4),
    {
    }
}

} // verus!
