// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessIdentifier Specification.
// This file contains spec functions for the ProcessIdentifier type.

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ProcessIdentifier.
#[verifier::ext_equal]
pub struct ProcessIdentifierView {
    /// The raw i32 value of the process identifier.
    pub value: int,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ProcessIdentifier {
    /// Spec function: returns the raw i32 value as an int.
    pub open spec fn spec_value(&self) -> int {
        self.value as int
    }

    /// Spec function: checks if the PID is non-negative (valid for usize conversion).
    pub open spec fn spec_is_non_negative(&self) -> bool {
        self.value >= 0
    }

    /// Spec function: checks if this is the kernel process identifier.
    pub open spec fn spec_is_kernel(&self) -> bool {
        self.value == Self::KERNEL_RAW
    }

    /// Spec function: checks if this is the init daemon process identifier.
    pub open spec fn spec_is_initd(&self) -> bool {
        self.value == 1
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Note
    ///
    /// ProcessIdentifier is a simple newtype wrapper around i32 with no
    /// structural invariants. Any i32 value is a valid ProcessIdentifier.
    /// The wf() predicate is therefore trivially true. Domain-specific
    /// constraints (e.g., PIDs must be non-negative in POSIX) are
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
    pub closed spec fn spec_to_ne_bytes(&self) -> [u8; 4];

    /// Spec function: abstract representation of from_ne_bytes result.
    ///
    /// # Note
    ///
    /// This is an abstract spec function representing the byte deserialization.
    /// It is uninterpreted but allows stating round-trip properties as axioms.
    pub closed spec fn spec_from_ne_bytes(bytes: [u8; 4]) -> int;

    /// Spec function: checks if value is within valid i32 range.
    pub open spec fn spec_in_i32_range(v: int) -> bool {
        i32::MIN as int <= v && v <= i32::MAX as int
    }

    /// Spec function: checks if value fits in non-negative i32 range (valid for usize).
    pub open spec fn spec_in_non_negative_i32_range(v: int) -> bool {
        0 <= v && v <= i32::MAX as int
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ProcessIdentifier {
    type V = ProcessIdentifierView;

    open spec fn view(&self) -> ProcessIdentifierView {
        ProcessIdentifierView { value: self.value as int }
    }
}

} // verus!
