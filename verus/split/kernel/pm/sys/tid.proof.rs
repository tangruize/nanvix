// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadIdentifier Proofs.
// This file contains proof lemmas for the ThreadIdentifier type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl ThreadIdentifier {
    /// Lemma: The KERNEL constant has value KERNEL_RAW (0).
    pub proof fn lemma_kernel_has_kernel_raw_value()
        ensures
            ThreadIdentifier::KERNEL.spec_value() == ThreadIdentifier::KERNEL_RAW as int,
            ThreadIdentifier::KERNEL.spec_is_kernel(),
            ThreadIdentifier::KERNEL.spec_is_non_negative(),
    {
    }

    /// Lemma: The INITD constant has value 1.
    pub proof fn lemma_initd_has_value_one()
        ensures
            ThreadIdentifier::INITD.spec_value() == 1,
            ThreadIdentifier::INITD.spec_is_initd(),
            ThreadIdentifier::INITD.spec_is_non_negative(),
    {
    }

    /// Lemma: A non-negative ThreadIdentifier can be converted to usize.
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
            (ThreadIdentifier { value: raw }).spec_value() == raw as int,
    {
    }

    /// Lemma: Spec-level value preservation for into_i32.
    pub proof fn lemma_into_i32_spec_value(&self)
        ensures
            self.value as int == self.spec_value(),
    {
    }

    /// Lemma: View equality implies value equality.
    pub proof fn lemma_view_equality(a: &ThreadIdentifier, b: &ThreadIdentifier)
        requires
            a@ == b@,
        ensures
            a.spec_value() == b.spec_value(),
    {
    }

    /// Lemma: Two ThreadIdentifiers with equal values have equal views.
    pub proof fn lemma_value_implies_view_equality(a: &ThreadIdentifier, b: &ThreadIdentifier)
        requires
            a.spec_value() == b.spec_value(),
        ensures
            a@ == b@,
    {
    }

    /// Axiom: Byte serialization round-trip preserves value.
    ///
    /// # Note
    ///
    /// This is an assumed property based on Rust's i32::to_ne_bytes/from_ne_bytes
    /// semantics. Verus cannot verify byte-level integer representation, so this
    /// property is documented as an axiom.
    ///
    /// Property: `from_ne_bytes(tid.to_ne_bytes()).spec_value() == tid.spec_value()`
    #[verifier::external_body]
    pub proof fn axiom_byte_roundtrip(tid: &ThreadIdentifier)
        ensures
            Self::spec_from_ne_bytes(tid.spec_to_ne_bytes()) == tid.spec_value(),
    {
    }

    /// Axiom: Byte decode-then-encode round-trip preserves bytes.
    ///
    /// # Note
    ///
    /// This is an assumed property based on Rust's i32::from_ne_bytes/to_ne_bytes
    /// semantics. Verus cannot verify byte-level integer representation, so this
    /// property is documented as an axiom.
    ///
    /// The `requires` clause ensures the decoded value is within i32 range, which
    /// guarantees the `v as i32` cast in the ensures clause is non-truncating.
    /// Use `axiom_from_ne_bytes_in_range` to establish this precondition, or
    /// prefer `lemma_byte_roundtrip_complete` which composes all byte axioms.
    ///
    /// Property: for any bytes, `from_ne_bytes(bytes).to_ne_bytes() == bytes`
    #[verifier::external_body]
    pub proof fn axiom_decode_encode_roundtrip(bytes: [u8; 4])
        requires
            i32::MIN as int <= Self::spec_from_ne_bytes(bytes) <= i32::MAX as int,
        ensures ({
            let v: int = Self::spec_from_ne_bytes(bytes);
            let tid: ThreadIdentifier = ThreadIdentifier { value: v as i32 };
            tid.spec_to_ne_bytes() == bytes
        }),
    {
    }

    /// Axiom: `spec_from_ne_bytes` always returns a value in i32 range.
    ///
    /// # Note
    ///
    /// This is an assumed property based on Rust's `i32::from_ne_bytes` semantics:
    /// every 4-byte array decodes to a valid i32 value. Without this axiom,
    /// downstream proofs cannot establish that a decoded TID value is within i32
    /// range, which could block proof composition for functions with i32-range
    /// preconditions. This axiom also makes `axiom_decode_encode_roundtrip` more
    /// usable, since it ensures the `v as i32` cast in that axiom is non-truncating.
    #[verifier::external_body]
    pub proof fn axiom_from_ne_bytes_in_range(bytes: [u8; 4])
        ensures
            i32::MIN as int <= Self::spec_from_ne_bytes(bytes) <= i32::MAX as int,
    {
    }

    /// Lemma: ThreadIdentifier has the same size as i32 (4 bytes).
    ///
    /// # Note
    ///
    /// This mirrors the original `assert_eq_size!(ThreadIdentifier, 4)` static
    /// assertion. Uses `external_body` because Verus cannot reason about
    /// `core::mem::size_of` directly.
    #[verifier::external_body]
    pub proof fn lemma_size_eq_i32()
        ensures
            core::mem::size_of::<ThreadIdentifier>() == core::mem::size_of::<i32>(),
            core::mem::size_of::<ThreadIdentifier>() == 4,
    {
    }

    /// Lemma: ThreadIdentifier has the same alignment as i32 (4 bytes).
    ///
    /// # Note
    ///
    /// This mirrors the original `assert_eq_align!(ThreadIdentifier, 4)` static
    /// assertion. Uses `external_body` because Verus cannot reason about
    /// `core::mem::align_of` directly.
    #[verifier::external_body]
    pub proof fn lemma_align_eq_i32()
        ensures
            core::mem::align_of::<ThreadIdentifier>() == core::mem::align_of::<i32>(),
            core::mem::align_of::<ThreadIdentifier>() == 4,
    {
    }

    /// Proof: Asserts layout invariants by invoking layout lemmas.
    ///
    /// # Note
    ///
    /// This is the verified equivalent of the original static assertions:
    /// - `assert_eq_size!(ThreadIdentifier, 4)`
    /// - `assert_eq_align!(ThreadIdentifier, 4)`
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

    /// Lemma: `lt(a, b)` iff `!ge(a, b)`.
    pub proof fn lemma_lt_iff_not_ge(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a.spec_value() < b.spec_value()) <==> !(a.spec_value() >= b.spec_value()),
    {
    }

    /// Lemma: `le(a, b)` iff `!gt(a, b)`.
    pub proof fn lemma_le_iff_not_gt(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a.spec_value() <= b.spec_value()) <==> !(a.spec_value() > b.spec_value()),
    {
    }

    /// Lemma: Equality is reflexive.
    pub proof fn lemma_eq_reflexive(a: &ThreadIdentifier)
        ensures
            a.spec_value() == a.spec_value(),
    {
    }

    /// Lemma: Ordering is transitive.
    pub proof fn lemma_lt_transitive(a: &ThreadIdentifier, b: &ThreadIdentifier, c: &ThreadIdentifier)
        requires
            a.spec_value() < b.spec_value(),
            b.spec_value() < c.spec_value(),
        ensures
            a.spec_value() < c.spec_value(),
    {
    }

    /// Lemma: Ordering is total — exactly one of `<`, `==`, `>` holds.
    pub proof fn lemma_ordering_total(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a.spec_value() < b.spec_value()) || (a.spec_value() == b.spec_value()) || (a.spec_value() > b.spec_value()),
            // Mutual exclusivity.
            !((a.spec_value() < b.spec_value()) && (a.spec_value() == b.spec_value())),
            !((a.spec_value() < b.spec_value()) && (a.spec_value() > b.spec_value())),
            !((a.spec_value() == b.spec_value()) && (a.spec_value() > b.spec_value())),
    {
    }

    /// Lemma: `spec_cmp` is consistent with `lt`/`eq`/`gt`.
    pub proof fn lemma_cmp_consistent(a: &ThreadIdentifier, b: &ThreadIdentifier)
        ensures
            (a.spec_cmp(b) == core::cmp::Ordering::Less) <==> (a.spec_value() < b.spec_value()),
            (a.spec_cmp(b) == core::cmp::Ordering::Equal) <==> (a.spec_value() == b.spec_value()),
            (a.spec_cmp(b) == core::cmp::Ordering::Greater) <==> (a.spec_value() > b.spec_value()),
    {
    }

    //==================================================================================================
    // Composite Byte Round-Trip Lemma
    //==================================================================================================

    /// Lemma: Complete byte round-trip properties.
    ///
    /// # Note
    ///
    /// Composes all three byte axioms into a single lemma that establishes
    /// both encode-decode and decode-encode round-trips along with the i32
    /// range constraint. Downstream consumers can invoke this single lemma
    /// instead of needing to know the axiom dependency order.
    pub proof fn lemma_byte_roundtrip_complete(tid: &ThreadIdentifier, bytes: [u8; 4])
        ensures
            // Encode-then-decode preserves value.
            Self::spec_from_ne_bytes(tid.spec_to_ne_bytes()) == tid.spec_value(),
            // Decoded value is always in i32 range.
            i32::MIN as int <= Self::spec_from_ne_bytes(bytes) <= i32::MAX as int,
            // Decode-then-encode preserves bytes.
            ({
                let v: int = Self::spec_from_ne_bytes(bytes);
                let reconstructed: ThreadIdentifier = ThreadIdentifier { value: v as i32 };
                reconstructed.spec_to_ne_bytes() == bytes
            }),
    {
        Self::axiom_byte_roundtrip(tid);
        Self::axiom_from_ne_bytes_in_range(bytes);
        Self::axiom_decode_encode_roundtrip(bytes);
    }
}

//==================================================================================================
// Module-Level Layout Assertion
//==================================================================================================

/// Proof: Module-level layout assertion mirroring the original static_assert! macros.
///
/// This ensures layout properties are checked as part of module verification,
/// not just available as callable lemmas.
proof fn assert_tid_layout()
    ensures
        core::mem::size_of::<ThreadIdentifier>() == 4,
        core::mem::align_of::<ThreadIdentifier>() == 4,
{
    ThreadIdentifier::assert_layout();
}

} // verus!
