// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadIdentifier Proofs (libs).
// This file contains proof lemmas for the ThreadIdentifier type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl ThreadIdentifier {
    /// Lemma: The KERNEL constant has value KERNEL_RAW (0).
    pub proof fn lemma_kernel_has_kernel_raw_value()
        ensures
            ThreadIdentifier::KERNEL@.value == ThreadIdentifier::KERNEL_RAW as int,
            ThreadIdentifier::KERNEL@.is_kernel(),
            ThreadIdentifier::KERNEL@.is_non_negative(),
            ThreadIdentifier::KERNEL.inv(),
    {
    }

    /// Lemma: The INITD constant has value 1.
    pub proof fn lemma_initd_has_value_one()
        ensures
            ThreadIdentifier::INITD@.value == 1,
            ThreadIdentifier::INITD@.is_initd(),
            ThreadIdentifier::INITD@.is_non_negative(),
            ThreadIdentifier::INITD.inv(),
    {
    }

    /// Lemma: A non-negative ThreadIdentifier can be converted to usize.
    pub proof fn lemma_non_negative_is_convertible_to_usize(&self)
        requires
            self@.is_non_negative(),
        ensures
            0 <= self@.value <= i32::MAX as int,
    {
    }

    /// Lemma: Spec-level value preservation for from_i32.
    pub proof fn lemma_from_i32_view_value(raw: i32)
        ensures
            (ThreadIdentifier { value: raw })@.value == raw as int,
    {
    }

    /// Lemma: View equality implies value equality.
    pub proof fn lemma_view_equality(a: &ThreadIdentifier, b: &ThreadIdentifier)
        requires
            a@ == b@,
        ensures
            a@.value == b@.value,
    {
    }

    /// Lemma: Two ThreadIdentifiers with equal values have equal views.
    pub proof fn lemma_value_implies_view_equality(a: &ThreadIdentifier, b: &ThreadIdentifier)
        requires
            a@.value == b@.value,
        ensures
            a@ == b@,
    {
    }

    /// Axiom: Byte serialization round-trip preserves value.
    ///
    /// # Note
    ///
    /// Uses external_body because Verus cannot reason about byte-level
    /// integer representation. Justified by Rust's i32::to_ne_bytes/from_ne_bytes.
    #[verifier::external_body]
    pub proof fn axiom_byte_roundtrip(tid: &ThreadIdentifier)
        ensures
            ThreadIdentifierView::from_ne_bytes_spec(tid@.to_ne_bytes_spec()) == tid@.value,
    {
    }

    /// Axiom: Byte decode-then-encode round-trip preserves bytes.
    ///
    /// # Note
    ///
    /// Uses external_body because Verus cannot reason about byte-level
    /// integer representation. Justified by Rust's from_ne_bytes/to_ne_bytes.
    #[verifier::external_body]
    pub proof fn axiom_decode_encode_roundtrip(bytes: [u8; 4])
        ensures ({
            let v: int = ThreadIdentifierView::from_ne_bytes_spec(bytes);
            let tid_view: ThreadIdentifierView = ThreadIdentifierView { value: v };
            tid_view.to_ne_bytes_spec() == bytes
        }),
    {
    }

    /// Axiom: from_ne_bytes_spec always returns a value in i32 range.
    ///
    /// # Note
    ///
    /// Uses external_body because Verus cannot reason about byte-level
    /// representation. Justified by Rust's i32::from_ne_bytes semantics.
    #[verifier::external_body]
    pub proof fn axiom_from_ne_bytes_in_range(bytes: [u8; 4])
        ensures
            i32::MIN as int <= ThreadIdentifierView::from_ne_bytes_spec(bytes) <= i32::MAX as int,
    {
    }

    /// Lemma: ThreadIdentifier has the same size as i32 (4 bytes).
    #[verifier::external_body]
    pub proof fn lemma_size_eq_i32()
        ensures
            core::mem::size_of::<ThreadIdentifier>() == core::mem::size_of::<i32>(),
            core::mem::size_of::<ThreadIdentifier>() == 4,
    {
    }

    /// Lemma: ThreadIdentifier has the same alignment as i32 (4 bytes).
    #[verifier::external_body]
    pub proof fn lemma_align_eq_i32()
        ensures
            core::mem::align_of::<ThreadIdentifier>() == core::mem::align_of::<i32>(),
            core::mem::align_of::<ThreadIdentifier>() == 4,
    {
    }

    /// Proof: Asserts layout invariants.
    pub proof fn assert_layout()
        ensures
            core::mem::size_of::<ThreadIdentifier>() == 4,
            core::mem::align_of::<ThreadIdentifier>() == 4,
    {
        Self::lemma_size_eq_i32();
        Self::lemma_align_eq_i32();
    }

    //==================================================================================================
    // Ordering Consistency Lemmas
    //==================================================================================================

    /// Lemma: lt iff not ge.
    pub proof fn lemma_lt_iff_not_ge(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a@.value < b@.value) <==> !(a@.value >= b@.value),
    {
    }

    /// Lemma: le iff not gt.
    pub proof fn lemma_le_iff_not_gt(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a@.value <= b@.value) <==> !(a@.value > b@.value),
    {
    }

    /// Lemma: Equality is reflexive.
    pub proof fn lemma_eq_reflexive(a: &ThreadIdentifier)
        ensures
            a@.value == a@.value,
    {
    }

    /// Lemma: Ordering is transitive.
    pub proof fn lemma_lt_transitive(a: &ThreadIdentifier, b: &ThreadIdentifier, c: &ThreadIdentifier)
        requires
            a@.value < b@.value,
            b@.value < c@.value,
        ensures
            a@.value < c@.value,
    {
    }

    /// Lemma: Ordering is total.
    pub proof fn lemma_ordering_total(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a@.value < b@.value) || (a@.value == b@.value) || (a@.value > b@.value),
            !((a@.value < b@.value) && (a@.value == b@.value)),
            !((a@.value < b@.value) && (a@.value > b@.value)),
            !((a@.value == b@.value) && (a@.value > b@.value)),
    {
    }

    //==================================================================================================
    // Composite Byte Round-Trip Lemma
    //==================================================================================================

    /// Lemma: Complete byte round-trip properties.
    pub proof fn lemma_byte_roundtrip_complete(tid: &ThreadIdentifier, bytes: [u8; 4])
        ensures
            ThreadIdentifierView::from_ne_bytes_spec(tid@.to_ne_bytes_spec()) == tid@.value,
            i32::MIN as int <= ThreadIdentifierView::from_ne_bytes_spec(bytes) <= i32::MAX as int,
            ({
                let v: int = ThreadIdentifierView::from_ne_bytes_spec(bytes);
                let view: ThreadIdentifierView = ThreadIdentifierView { value: v };
                view.to_ne_bytes_spec() == bytes
            }),
    {
        Self::axiom_byte_roundtrip(tid);
        Self::axiom_from_ne_bytes_in_range(bytes);
        Self::axiom_decode_encode_roundtrip(bytes);
    }
}

} // verus!
