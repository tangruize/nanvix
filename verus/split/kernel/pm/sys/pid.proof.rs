// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessIdentifier Proofs
//!
//! This file contains proof lemmas for the ProcessIdentifier type.

use vstd::prelude::*;

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

    /// Lemma: Conversion from i32 preserves value.
    pub proof fn lemma_from_i32_preserves_value(raw: i32)
        ensures
            ProcessIdentifier::from_i32(raw).spec_value() == raw as int,
    {
    }

    /// Lemma: Conversion to i32 preserves value.
    pub proof fn lemma_to_i32_preserves_value(&self)
        ensures
            self.into_i32() as int == self.spec_value(),
    {
    }

    /// Lemma: Round-trip conversion from i32 and back.
    pub proof fn lemma_roundtrip_i32(raw: i32)
        ensures
            ProcessIdentifier::from_i32(raw).into_i32() == raw,
    {
    }

    /// Lemma: View equality implies structural equality.
    pub proof fn lemma_view_equality(a: &ProcessIdentifier, b: &ProcessIdentifier)
        requires
            a@ == b@,
        ensures
            a.spec_value() == b.spec_value(),
    {
    }
}

} // verus!
