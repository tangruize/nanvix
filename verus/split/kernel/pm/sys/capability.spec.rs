// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capability Specification.
// This file contains spec functions for the Capability type.

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a Capability, represented as its integer discriminant.
#[verifier::ext_equal]
pub struct CapabilityView {
    /// The discriminant value (0..=4 for valid capabilities).
    pub value: int,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Capability {
    /// Spec function: returns the discriminant value as an int.
    pub open spec fn spec_discriminant(&self) -> int {
        match *self {
            Capability::ExceptionControl => 0,
            Capability::InterruptControl => 1,
            Capability::IoManagement => 2,
            Capability::MemoryManagement => 3,
            Capability::ProcessManagement => 4,
        }
    }

    /// Spec function: checks whether a u32 value maps to a valid capability.
    pub open spec fn spec_is_valid_discriminant(value: int) -> bool {
        0 <= value && value <= 4
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Note
    ///
    /// A Capability is always well-formed because all enum variants are valid.
    /// The discriminant is always in [0, 4].
    pub open spec fn wf(&self) -> bool {
        Self::spec_is_valid_discriminant(self.spec_discriminant())
    }

    /// Spec function: maps a valid discriminant to the expected capability variant.
    pub open spec fn spec_from_discriminant(value: int) -> Capability
        requires Self::spec_is_valid_discriminant(value)
    {
        if value == 0 {
            Capability::ExceptionControl
        } else if value == 1 {
            Capability::InterruptControl
        } else if value == 2 {
            Capability::IoManagement
        } else if value == 3 {
            Capability::MemoryManagement
        } else {
            Capability::ProcessManagement
        }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Capability {
    type V = CapabilityView;

    open spec fn view(&self) -> CapabilityView {
        CapabilityView { value: self.spec_discriminant() }
    }
}

} // verus!
