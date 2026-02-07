// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capability Proofs.
// This file contains proof lemmas for the Capability type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl Capability {
    /// Lemma: All capability variants are well-formed.
    pub proof fn lemma_all_variants_wf()
        ensures
            Capability::ExceptionControl.wf(),
            Capability::InterruptControl.wf(),
            Capability::IoManagement.wf(),
            Capability::MemoryManagement.wf(),
            Capability::ProcessManagement.wf(),
    {
    }

    /// Lemma: Each variant has a unique discriminant.
    pub proof fn lemma_discriminants_unique()
        ensures
            Capability::ExceptionControl.spec_discriminant() == 0,
            Capability::InterruptControl.spec_discriminant() == 1,
            Capability::IoManagement.spec_discriminant() == 2,
            Capability::MemoryManagement.spec_discriminant() == 3,
            Capability::ProcessManagement.spec_discriminant() == 4,
    {
    }

    /// Lemma: Any Capability instance is well-formed (discriminant in [0, 4]).
    pub proof fn lemma_wf(&self)
        ensures
            self.wf(),
    {
        // Follows from exhaustive match in spec_discriminant.
    }

    /// Lemma: Discriminants are mutually exclusive — no two variants share a discriminant.
    pub proof fn lemma_discriminants_disjoint(a: &Capability, b: &Capability)
        requires
            a.spec_discriminant() == b.spec_discriminant(),
        ensures
            *a == *b,
    {
    }

    /// Lemma: Round-trip — try_from_u32 for a valid discriminant returns a capability
    /// whose discriminant matches the input.
    pub proof fn lemma_try_from_roundtrip(v: u32)
        requires
            Capability::spec_is_valid_discriminant(v as int),
        ensures
            Capability::spec_from_discriminant(v as int).spec_discriminant() == v as int,
    {
    }

    /// Lemma: spec_from_discriminant is a left inverse of spec_discriminant.
    pub proof fn lemma_from_discriminant_inverse(&self)
        ensures
            Capability::spec_from_discriminant(self.spec_discriminant()) == *self,
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
            0 <= self.spec_discriminant() <= 4,
    {
    }

    /// Lemma: Total coverage — exactly 5 valid discriminants exist.
    pub proof fn lemma_valid_discriminant_count()
        ensures
            forall|v: int| Capability::spec_is_valid_discriminant(v) <==> (0 <= v <= 4),
    {
    }
}

} // verus!
