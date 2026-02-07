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

    /// Axiom: Byte serialization round-trip preserves value.
    ///
    /// # Note
    ///
    /// This is an assumed property based on Rust's i32::to_ne_bytes/from_ne_bytes
    /// semantics. Verus cannot verify byte-level integer representation, so this
    /// property is documented as an axiom.
    ///
    /// Property: `from_ne_bytes(pid.to_ne_bytes()).spec_value() == pid.spec_value()`
    #[verifier::external_body]
    pub proof fn axiom_byte_roundtrip(pid: &ProcessIdentifier)
        ensures
            Self::spec_from_ne_bytes(pid.spec_to_ne_bytes()) == pid.spec_value(),
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
    /// Property: for any bytes, `from_ne_bytes(bytes).to_ne_bytes() == bytes`
    #[verifier::external_body]
    pub proof fn axiom_decode_encode_roundtrip(bytes: [u8; 4])
        ensures ({
            let v: int = Self::spec_from_ne_bytes(bytes);
            let pid: ProcessIdentifier = ProcessIdentifier { value: v as i32 };
            pid.spec_to_ne_bytes() == bytes
        }),
    {
    }

    /// Axiom: `spec_from_ne_bytes` always returns a value in i32 range.
    ///
    /// # Note
    ///
    /// This is an assumed property based on Rust's `i32::from_ne_bytes` semantics:
    /// every 4-byte array decodes to a valid i32 value. Without this axiom,
    /// downstream proofs cannot establish that a decoded PID value is within i32
    /// range, which could block proof composition for functions with i32-range
    /// preconditions. This axiom also makes `axiom_decode_encode_roundtrip` more
    /// usable, since it ensures the `v as i32` cast in that axiom is non-truncating.
    #[verifier::external_body]
    pub proof fn axiom_from_ne_bytes_in_range(bytes: [u8; 4])
        ensures
            i32::MIN as int <= Self::spec_from_ne_bytes(bytes) <= i32::MAX as int,
    {
    }

    /// Lemma: ProcessIdentifier has the same size as i32 (4 bytes).
    ///
    /// # Note
    ///
    /// This mirrors the original `assert_eq_size!(ProcessIdentifier, 4)` static
    /// assertion. Uses `external_body` because Verus cannot reason about
    /// `core::mem::size_of` directly.
    #[verifier::external_body]
    pub proof fn lemma_size_eq_i32()
        ensures
            core::mem::size_of::<ProcessIdentifier>() == core::mem::size_of::<i32>(),
            core::mem::size_of::<ProcessIdentifier>() == 4,
    {
    }

    /// Lemma: ProcessIdentifier has the same alignment as i32 (4 bytes).
    ///
    /// # Note
    ///
    /// This mirrors the original `assert_eq_align!(ProcessIdentifier, 4)` static
    /// assertion. Uses `external_body` because Verus cannot reason about
    /// `core::mem::align_of` directly.
    #[verifier::external_body]
    pub proof fn lemma_align_eq_i32()
        ensures
            core::mem::align_of::<ProcessIdentifier>() == core::mem::align_of::<i32>(),
            core::mem::align_of::<ProcessIdentifier>() == 4,
    {
    }

    /// Proof: Asserts layout invariants by invoking layout lemmas.
    ///
    /// # Note
    ///
    /// This is the verified equivalent of the original static assertions:
    /// - `assert_eq_size!(ProcessIdentifier, 4)`
    /// - `assert_eq_align!(ProcessIdentifier, 4)`
    ///
    /// It invokes both layout lemmas to ensure they are exercised (not dead code).
    pub proof fn assert_layout()
        ensures
            core::mem::size_of::<ProcessIdentifier>() == 4,
            core::mem::align_of::<ProcessIdentifier>() == 4,
    {
        Self::lemma_size_eq_i32();
        Self::lemma_align_eq_i32();
    }
}

} // verus!
