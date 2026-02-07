// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessIdentifier Specification
//!
//! This file contains spec functions for the ProcessIdentifier type.

use vstd::prelude::*;

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

    /// Spec function: well-formedness (always true for ProcessIdentifier).
    pub open spec fn wf(&self) -> bool {
        true
    }

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
