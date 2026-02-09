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
        self.0
    }

    /// Spec function: checks whether a specific capability bit is set.
    pub open spec fn spec_has(&self, cap: Capability) -> bool {
        (self.0 & (1u8 << cap.spec_discriminant())) != 0u8
    }

    /// Spec function: returns the bitfield after setting a capability bit.
    pub open spec fn spec_set(&self, cap: Capability) -> u8 {
        (self.0 | (1u8 << cap.spec_discriminant())) as u8
    }

    /// Spec function: returns the bitfield after clearing a capability bit.
    pub open spec fn spec_clear(&self, cap: Capability) -> u8 {
        (self.0 & !(1u8 << cap.spec_discriminant())) as u8
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Note
    ///
    /// A Capabilities value is always well-formed because only bits 0..4
    /// are semantically meaningful (corresponding to the 5 capability variants),
    /// but any u8 value is a valid bitfield representation.
    pub open spec fn wf(&self) -> bool {
        true
    }

    /// Spec function: checks whether only valid capability bits are set.
    ///
    /// # Note
    ///
    /// Valid bits are 0..=4, corresponding to the 5 Capability variants.
    /// The upper 3 bits (5, 6, 7) should not be set.
    pub open spec fn spec_only_valid_bits(&self) -> bool {
        self.0 & 0b1110_0000u8 == 0u8
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Capabilities {
    type V = CapabilitiesView;

    open spec fn view(&self) -> CapabilitiesView {
        CapabilitiesView { bits: self.0 }
    }
}

} // verus!
