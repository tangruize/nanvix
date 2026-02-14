// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capability Specification (libs).
// This file contains CapabilityView, View trait, invariant, and spec functions.
// Methodology: Step 1 (abstraction), Step 2 (invariant), Step 3 (public specs).

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a Capability.
///
/// # Description
///
/// Uses `int` instead of `u32` for abstract reasoning (methodology Step 1).
/// The discriminant value maps to enum variants: 0..=4.
#[verifier::ext_equal]
pub struct CapabilityView {
    /// The abstract discriminant value.
    pub value: int,
}

//==================================================================================================
// CapabilityView Spec Functions
//==================================================================================================

impl CapabilityView {
    /// Whether the discriminant corresponds to a valid capability variant.
    pub open spec fn is_valid(&self) -> bool {
        0 <= self.value && self.value <= 4
    }

    /// Whether a given int is a valid capability discriminant.
    pub open spec fn is_valid_discriminant(v: int) -> bool {
        0 <= v && v <= 4
    }
}

//==================================================================================================
// Capability Spec Functions
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

    /// Spec function: checks whether an int value maps to a valid capability.
    pub open spec fn spec_is_valid_discriminant(value: int) -> bool {
        0 <= value && value <= 4
    }

    /// Spec function: maps a valid discriminant to the expected capability variant.
    ///
    /// For out-of-range inputs, the result is unspecified (`arbitrary()`).
    pub open spec fn spec_from_discriminant(value: int) -> Capability
        recommends Self::spec_is_valid_discriminant(value)
    {
        if value == 0 {
            Capability::ExceptionControl
        } else if value == 1 {
            Capability::InterruptControl
        } else if value == 2 {
            Capability::IoManagement
        } else if value == 3 {
            Capability::MemoryManagement
        } else if value == 4 {
            Capability::ProcessManagement
        } else {
            arbitrary()
        }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Capability {
    type V = CapabilityView;

    // Closed per methodology Step 1: hides implementation internals from users.
    closed spec fn view(&self) -> CapabilityView {
        CapabilityView { value: self.spec_discriminant() }
    }
}

//==================================================================================================
// Invariant
//==================================================================================================

impl Capability {
    /// Invariant for Capability (methodology Step 2).
    ///
    /// # Description
    ///
    /// Capability is a simple enum with five variants. All variants are
    /// well-formed by construction (discriminants are always in [0, 4]),
    /// so the invariant is trivially true.
    pub closed spec fn inv(&self) -> bool {
        Self::spec_is_valid_discriminant(self.spec_discriminant())
    }
}

} // verus!
