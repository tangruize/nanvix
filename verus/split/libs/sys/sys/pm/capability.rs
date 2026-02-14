// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Capability Implementation (libs)
//!
//! A type that represents a process capability.
//!
//! ## Verified Properties
//!
//! - All enum variants have unique discriminants (0..=4).
//! - All variants satisfy the invariant.
//! - Conversion from u32 succeeds iff the value is in [0, 4].
//! - Conversion from u32 preserves the discriminant value.
//! - Round-trip: try_from_u32(cap.to_u32()) == Ok(cap).
//! - Error paths return `ErrorCode::InvalidArgument`.
//!
//! ## Methodology Compliance
//!
//! - Step 1: `CapabilityView` uses `int` (not `u32`). `view()` is `closed`.
//! - Step 2: `inv()` is `pub closed spec fn`. True when discriminant is in [0, 4].
//! - Step 3: Public specs use `self@.value` (view), not `self.field` (impl).
//!           No `pub` spec functions on `impl Capability` beyond `inv()` and `view()`.
//! - Step 5: No `assume()`, `admit()`, or unjustified `external_body`.
//!
//! ## Trust Boundary
//!
//! No `external_body` or `assume` is used in this module.

use crate::libs::error::{
    Error,
    ErrorCode,
};
use vstd::prelude::*;

// Include specifications.
include!("capability.spec.rs");

// Include proofs.
include!("capability.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A type that represents a capability.
///
/// # Description
///
/// Capability represents a specific type of system-level permission that can be
/// granted to a process. Each variant corresponds to a distinct area of kernel
/// functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Exception control.
    ExceptionControl,
    /// Interrupt control.
    InterruptControl,
    /// I/O management.
    IoManagement,
    /// Memory management.
    MemoryManagement,
    /// Process management.
    ProcessManagement,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Capability {
    /// Error message for invalid capability conversions.
    pub const PARSE_ERROR_MESSAGE: &'static str = "invalid capability";

    /// Converts a u32 value to a Capability.
    ///
    /// # Parameters
    ///
    /// - `value`: The raw u32 value.
    ///
    /// # Returns
    ///
    /// On success, the corresponding Capability variant.
    /// On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value does not correspond to a valid capability.
    pub fn try_from_u32(value: u32) -> (result: Result<Capability, Error>)
        ensures
            result is Ok ==> {
                &&& CapabilityView::is_valid_discriminant(value as int)
                &&& result->Ok_0@.value == value as int
                &&& result->Ok_0 == Self::spec_from_discriminant(value as int)
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !CapabilityView::is_valid_discriminant(value as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        match value {
            0u32 => Ok(Capability::ExceptionControl),
            1u32 => Ok(Capability::InterruptControl),
            2u32 => Ok(Capability::IoManagement),
            3u32 => Ok(Capability::MemoryManagement),
            4u32 => Ok(Capability::ProcessManagement),
            _ => Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE)),
        }
    }

    /// Converts a Capability to its u32 discriminant value.
    ///
    /// # Note
    ///
    /// This function is a verification auxiliary not present in the original source.
    /// It enables round-trip proofs for discriminant conversion.
    ///
    /// # Returns
    ///
    /// The discriminant value as u32.
    pub fn to_u32(&self) -> (result: u32)
        requires
            self.inv(),
        ensures
            result as int == self@.value,
            CapabilityView::is_valid_discriminant(result as int),
    {
        match *self {
            Capability::ExceptionControl => 0u32,
            Capability::InterruptControl => 1u32,
            Capability::IoManagement => 2u32,
            Capability::MemoryManagement => 3u32,
            Capability::ProcessManagement => 4u32,
        }
    }
}

//==================================================================================================
// Trait Implementations
//==================================================================================================

impl TryFrom<u32> for Capability {
    type Error = Error;

    /// Converts a u32 value to a Capability via the verified `try_from_u32` method.
    fn try_from(value: u32) -> (result: Result<Self, Self::Error>)
        ensures
            result is Ok ==> {
                &&& CapabilityView::is_valid_discriminant(value as int)
                &&& result->Ok_0@.value == value as int
                &&& result->Ok_0 == Self::spec_from_discriminant(value as int)
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !CapabilityView::is_valid_discriminant(value as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        Self::try_from_u32(value)
    }
}

} // verus!
