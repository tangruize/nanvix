// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capabilities Specification.
// This file contains spec functions for the Capabilities type.

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a Capabilities, represented as a set of active capability bits.
#[verifier::ext_equal]
pub struct CapabilitiesView {
    /// The raw bitfield value.
    pub bits: u8,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Capabilities {
    /// Spec function: returns the raw bitfield value.
    pub open spec fn spec_bits(&self) -> u8 {
        self.bits
    }

    /// Spec function: returns the bitmask for a given capability.
    ///
    /// Maps each variant to its corresponding single-bit mask:
    /// ExceptionControl -> 0x01, InterruptControl -> 0x02, IoManagement -> 0x04,
    /// MemoryManagement -> 0x08, ProcessManagement -> 0x10.
    ///
    /// # Note
    ///
    /// Uses explicit match rather than `1 << discriminant` to avoid dependence
    /// on enum layout or `#[repr]` annotations. The equivalence to the original
    /// source's shift-based formula is proven by `lemma_mask_matches_discriminant`.
    pub open spec fn spec_mask(cap: Capability) -> u8 {
        match cap {
            Capability::ExceptionControl => 1u8,
            Capability::InterruptControl => 2u8,
            Capability::IoManagement => 4u8,
            Capability::MemoryManagement => 8u8,
            Capability::ProcessManagement => 16u8,
        }
    }

    /// Spec function: maps a discriminant value to its power-of-2 bitmask.
    ///
    /// This is the mathematical `2^d` for valid discriminants (0..=4),
    /// corresponding to `1u8 << d` in the original source. It bridges the
    /// gap between the discriminant-based formula and the explicit mask values.
    pub open spec fn spec_pow2_mask(d: int) -> u8
        recommends 0 <= d <= 4
    {
        if d == 0 { 1u8 }
        else if d == 1 { 2u8 }
        else if d == 2 { 4u8 }
        else if d == 3 { 8u8 }
        else if d == 4 { 16u8 }
        else { arbitrary() }
    }

    /// Spec function: checks whether a specific capability bit is set.
    pub open spec fn spec_has(&self, cap: Capability) -> bool {
        (self.bits & Self::spec_mask(cap)) != 0u8
    }

    /// Spec function: returns the bitfield after setting a capability bit.
    pub open spec fn spec_set(&self, cap: Capability) -> u8 {
        (self.bits | Self::spec_mask(cap)) as u8
    }

    /// Spec function: returns the bitfield after clearing a capability bit.
    pub open spec fn spec_clear(&self, cap: Capability) -> u8 {
        (self.bits & !Self::spec_mask(cap)) as u8
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Note
    ///
    /// A Capabilities value is well-formed when only valid capability bits
    /// (0..=4) are set. The upper 3 bits (5, 6, 7) must not be set.
    /// All constructors (`new`, `default`) produce well-formed values, and
    /// `set`/`clear` preserve well-formedness (proven as conditional postcondition).
    ///
    /// This is the module-level invariant. Values constructed via the public API
    /// always satisfy `wf()`. The `pub bits` field allows constructing non-`wf`
    /// values, but such values are outside the intended usage; the verification
    /// guarantees correctness for the API-reachable state space.
    pub open spec fn wf(&self) -> bool {
        self.bits & 0b1110_0000u8 == 0u8
    }

    /// Spec function: predicate asserting that every valid capability mask
    /// only uses bits in the lower 5 positions (0..=4).
    ///
    /// # Note
    ///
    /// This is a consequence of the closed-world enum: since all 5 discriminants
    /// are in [0, 4], all masks are powers of 2 up to 2^4 = 16, and none set
    /// bits 5, 6, or 7. This connects `spec_mask` to `wf()`.
    pub open spec fn spec_mask_is_valid(cap: Capability) -> bool {
        Self::spec_mask(cap) & 0b1110_0000u8 == 0u8
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Capabilities {
    type V = CapabilitiesView;

    open spec fn view(&self) -> CapabilitiesView {
        CapabilitiesView { bits: self.bits }
    }
}

} // verus!
