// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessIdentifier Proofs.
// This file contains proof lemmas for the ProcessIdentifier type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl ProcessIdentifier {
    /// Lemma: The KERNEL constant has value KERNEL_RAW (0).
    pub proof fn lemma_kernel_has_kernel_raw_value()
        ensures
            ProcessIdentifier::KERNEL.spec_value() == ProcessIdentifier::KERNEL_RAW as int,
            ProcessIdentifier::KERNEL.spec_is_kernel(),
            ProcessIdentifier::KERNEL.spec_is_non_negative(),
    {
    }

    /// Lemma: The INITD constant has value 1.
    pub proof fn lemma_initd_has_value_one()
        ensures
            ProcessIdentifier::INITD.spec_value() == 1,
            ProcessIdentifier::INITD.spec_is_initd(),
            ProcessIdentifier::INITD.spec_is_non_negative(),
    {
    }

    /// Lemma: A non-negative ProcessIdentifier can be converted to usize.
    pub proof fn lemma_non_negative_is_convertible_to_usize(&self)
        requires
            self.spec_is_non_negative(),
        ensures
            0 <= self.spec_value() <= i32::MAX as int,
    {
    }

    /// Lemma: Spec-level value preservation for from_i32.
    pub proof fn lemma_from_i32_spec_value(raw: i32)
        ensures
            (ProcessIdentifier { value: raw }).spec_value() == raw as int,
    {
    }

    /// Lemma: Spec-level value preservation for into_i32.
    pub proof fn lemma_into_i32_spec_value(&self)
        ensures
            self.value as int == self.spec_value(),
    {
    }

    /// Lemma: View equality implies value equality.
    pub proof fn lemma_view_equality(a: &ProcessIdentifier, b: &ProcessIdentifier)
        requires
            a@ == b@,
        ensures
            a.spec_value() == b.spec_value(),
    {
    }

    /// Lemma: Two ProcessIdentifiers with equal values have equal views.
    pub proof fn lemma_value_implies_view_equality(a: &ProcessIdentifier, b: &ProcessIdentifier)
        requires
            a.spec_value() == b.spec_value(),
        ensures
            a@ == b@,
    {
    }
}

} // verus!
