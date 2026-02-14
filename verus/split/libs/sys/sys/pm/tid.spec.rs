// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadIdentifier Specification (libs).
// This file contains ThreadIdentifierView, View trait, and invariant.
// Methodology: Step 1 (abstraction), Step 2 (invariant), Step 3 (public specs).

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ThreadIdentifier.
///
/// # Description
///
/// Uses `int` instead of `i32` for abstract reasoning (methodology Step 1).
/// Implementation-only fields are not exposed.
#[verifier::ext_equal]
pub struct ThreadIdentifierView {
    /// The abstract value of the thread identifier.
    pub value: int,
}

//==================================================================================================
// ThreadIdentifierView Spec Functions
//==================================================================================================

impl ThreadIdentifierView {
    /// Whether the value is non-negative.
    pub open spec fn is_non_negative(&self) -> bool {
        self.value >= 0
    }

    /// Whether this is the kernel thread identifier (value 0).
    pub open spec fn is_kernel(&self) -> bool {
        self.value == 0
    }

    /// Whether this is the init daemon thread identifier (value 1).
    pub open spec fn is_initd(&self) -> bool {
        self.value == 1
    }

    /// Whether a value is within the i32 range.
    pub open spec fn in_i32_range(v: int) -> bool {
        i32::MIN as int <= v && v <= i32::MAX as int
    }

    /// Whether a value fits in the non-negative i32 range.
    pub open spec fn in_non_negative_i32_range(v: int) -> bool {
        0 <= v && v <= i32::MAX as int
    }

    /// Abstract byte serialization.
    pub uninterp spec fn to_ne_bytes_spec(&self) -> [u8; 4];

    /// Abstract byte deserialization.
    pub uninterp spec fn from_ne_bytes_spec(bytes: [u8; 4]) -> int;
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ThreadIdentifier {
    type V = ThreadIdentifierView;

    // Closed per methodology Step 1: hides implementation internals from users.
    closed spec fn view(&self) -> ThreadIdentifierView {
        ThreadIdentifierView { value: self.value as int }
    }
}

//==================================================================================================
// Invariant
//==================================================================================================

impl ThreadIdentifier {
    /// Invariant for ThreadIdentifier (methodology Step 2).
    ///
    /// # Description
    ///
    /// ThreadIdentifier is a simple newtype wrapper around i32 with no
    /// structural invariants. Any i32 value is valid, so inv is trivially true.
    /// Domain-specific constraints (e.g., non-negative TIDs) are application-level.
    pub closed spec fn inv(&self) -> bool {
        true
    }
}

} // verus!
