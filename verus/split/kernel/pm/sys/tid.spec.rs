// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadIdentifier Specification.
// This file contains spec functions for the ThreadIdentifier type.

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ThreadIdentifier.
#[verifier::ext_equal]
pub struct ThreadIdentifierView {
    /// The raw i32 value of the thread identifier.
    pub value: int,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ThreadIdentifier {
    /// Spec function: returns the raw i32 value as an int.
    pub open spec fn spec_value(&self) -> int {
        self.value as int
    }

    /// Spec function: checks if the TID is non-negative (valid for usize conversion).
    pub open spec fn spec_is_non_negative(&self) -> bool {
        self.value >= 0
    }

    /// Spec function: checks if this is the kernel thread identifier.
    pub open spec fn spec_is_kernel(&self) -> bool {
        self.value == Self::KERNEL_RAW
    }

    /// Spec function: checks if this is the init daemon thread identifier.
    pub open spec fn spec_is_initd(&self) -> bool {
        self.value == 1
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Note
    ///
    /// ThreadIdentifier is a simple newtype wrapper around i32 with no
    /// structural invariants. Any i32 value is a valid ThreadIdentifier.
    /// The wf() predicate is therefore trivially true. Domain-specific
    /// constraints (e.g., TIDs must be non-negative) are
    /// application-level concerns, not type invariants.
    pub open spec fn wf(&self) -> bool {
        true
    }

    /// Spec function: abstract representation of to_ne_bytes result.
    ///
    /// # Note
    ///
    /// This is an abstract spec function representing the byte serialization.
    /// It is uninterpreted but allows stating round-trip properties as axioms.
    pub uninterp spec fn spec_to_ne_bytes(&self) -> [u8; 4];

    /// Spec function: abstract representation of from_ne_bytes result.
    ///
    /// # Note
    ///
    /// This is an abstract spec function representing the byte deserialization.
    /// It is uninterpreted but allows stating round-trip properties as axioms.
    pub uninterp spec fn spec_from_ne_bytes(bytes: [u8; 4]) -> int;

    /// Spec function: checks if value is within valid i32 range.
    pub open spec fn spec_in_i32_range(v: int) -> bool {
        i32::MIN as int <= v && v <= i32::MAX as int
    }

    /// Spec function: checks if value fits in non-negative i32 range (valid for usize).
    pub open spec fn spec_in_non_negative_i32_range(v: int) -> bool {
        0 <= v && v <= i32::MAX as int
    }

    /// Spec function: compares two ThreadIdentifiers, returning an Ordering.
    pub open spec fn spec_cmp(&self, other: &ThreadIdentifier) -> core::cmp::Ordering {
        if self.spec_value() < other.spec_value() {
            core::cmp::Ordering::Less
        } else if self.spec_value() > other.spec_value() {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ThreadIdentifier {
    type V = ThreadIdentifierView;

    open spec fn view(&self) -> ThreadIdentifierView {
        ThreadIdentifierView { value: self.value as int }
    }
}

} // verus!
