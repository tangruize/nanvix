// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessIdentifier Implementation
//!
//! A type that represents a process identifier (PID).
//!
//! ## Verified Properties
//!
//! - Constants KERNEL and INITD have expected values (0 and 1).
//! - Conversion from/to i32 preserves value.
//! - Conversion to/from other integer types is safe when in range.
//! - Byte serialization round-trips correctly.

use crate::libs::error::{
    Error,
    ErrorCode,
};
use vstd::prelude::*;

// Include specifications.
include!("pid.spec.rs");

// Include proofs.
include!("pid.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A type that represents a process identifier.
///
/// # Description
///
/// ProcessIdentifier wraps an i32 value that uniquely identifies a process.
/// Special constants:
/// - KERNEL (0): The kernel process.
/// - INITD (1): The init daemon process.
///
/// # Note on Verification
///
/// The `value` field is `pub(crate)` for Verus spec reasoning. The original type
/// uses a tuple struct with private field. Verified code should use accessor methods
/// (`into_i32`, `from_i32`) rather than direct field access.
#[derive(Clone, Copy)]
pub struct ProcessIdentifier {
    /// The raw i32 value of the process identifier.
    /// Note: pub(crate) for Verus spec access; prefer using accessor methods.
    pub value: i32,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessIdentifier {
    /// Raw identifier for the kernel process.
    pub const KERNEL_RAW: i32 = 0;

    /// Identifier of the kernel process.
    pub const KERNEL: ProcessIdentifier = ProcessIdentifier { value: Self::KERNEL_RAW };

    /// Identifier of the init daemon process.
    pub const INITD: ProcessIdentifier = ProcessIdentifier { value: 1 };

    /// Creates a new ProcessIdentifier from an i32 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw i32 value.
    ///
    /// # Returns
    ///
    /// A new ProcessIdentifier with the given value.
    #[inline]
    pub fn from_i32(raw: i32) -> (result: ProcessIdentifier)
        ensures
            result.spec_value() == raw as int,
            result@ == (ProcessIdentifierView { value: raw as int }),
    {
        ProcessIdentifier { value: raw }
    }

    /// Converts the ProcessIdentifier to an i32 value.
    ///
    /// # Returns
    ///
    /// The raw i32 value.
    #[inline]
    pub fn into_i32(self) -> (result: i32)
        ensures
            result as int == self.spec_value(),
    {
        self.value
    }

    /// Converts the ProcessIdentifier to an isize value.
    ///
    /// # Returns
    ///
    /// The value as isize.
    #[inline]
    pub fn into_isize(self) -> (result: isize)
        ensures
            result as int == self.spec_value(),
    {
        self.value as isize
    }

    /// Converts the ProcessIdentifier to an i64 value.
    ///
    /// # Returns
    ///
    /// The value as i64.
    #[inline]
    pub fn into_i64(self) -> (result: i64)
        ensures
            result as int == self.spec_value(),
    {
        self.value as i64
    }

    /// Tries to convert the ProcessIdentifier to a usize value.
    ///
    /// # Returns
    ///
    /// On success, the value as usize. On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub fn try_into_usize(self) -> (result: Result<usize, Error>)
        ensures
            result is Ok ==> {
                &&& self.spec_is_non_negative()
                &&& result->Ok_0 as int == self.spec_value()
            },
            result is Err ==> !self.spec_is_non_negative(),
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(self.value as usize)
    }

    /// Tries to convert the ProcessIdentifier to a u32 value.
    ///
    /// # Returns
    ///
    /// On success, the value as u32. On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub fn try_into_u32(self) -> (result: Result<u32, Error>)
        ensures
            result is Ok ==> {
                &&& self.spec_is_non_negative()
                &&& result->Ok_0 as int == self.spec_value()
            },
            result is Err ==> !self.spec_is_non_negative(),
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(self.value as u32)
    }

    /// Tries to convert the ProcessIdentifier to a u64 value.
    ///
    /// # Returns
    ///
    /// On success, the value as u64. On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub fn try_into_u64(self) -> (result: Result<u64, Error>)
        ensures
            result is Ok ==> {
                &&& self.spec_is_non_negative()
                &&& result->Ok_0 as int == self.spec_value()
            },
            result is Err ==> !self.spec_is_non_negative(),
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(self.value as u64)
    }

    /// Creates a ProcessIdentifier from an isize value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw isize value.
    ///
    /// # Returns
    ///
    /// On success, a ProcessIdentifier with the given value.
    /// On failure, an error if the value is out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is outside the i32 range.
    pub fn try_from_isize(raw: isize) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& (i32::MIN as int) <= (raw as int) <= (i32::MAX as int)
                &&& result->Ok_0.spec_value() == raw as int
            },
            result is Err ==> (raw as int) < (i32::MIN as int) || (raw as int) > (i32::MAX as int),
    {
        if raw < i32::MIN as isize || raw > i32::MAX as isize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(ProcessIdentifier { value: raw as i32 })
    }

    /// Creates a ProcessIdentifier from an i64 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw i64 value.
    ///
    /// # Returns
    ///
    /// On success, a ProcessIdentifier with the given value.
    /// On failure, an error if the value is out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is outside the i32 range.
    pub fn try_from_i64(raw: i64) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& (i32::MIN as int) <= (raw as int) <= (i32::MAX as int)
                &&& result->Ok_0.spec_value() == raw as int
            },
            result is Err ==> (raw as int) < (i32::MIN as int) || (raw as int) > (i32::MAX as int),
    {
        if raw < i32::MIN as i64 || raw > i32::MAX as i64 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(ProcessIdentifier { value: raw as i32 })
    }

    /// Creates a ProcessIdentifier from a usize value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw usize value.
    ///
    /// # Returns
    ///
    /// On success, a ProcessIdentifier with the given value.
    /// On failure, an error if the value exceeds i32::MAX.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub fn try_from_usize(raw: usize) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& (raw as int) <= (i32::MAX as int)
                &&& result->Ok_0.spec_value() == raw as int
                &&& result->Ok_0.spec_is_non_negative()
            },
            result is Err ==> (raw as int) > (i32::MAX as int),
    {
        if raw > i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(ProcessIdentifier { value: raw as i32 })
    }

    /// Creates a ProcessIdentifier from a u32 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw u32 value.
    ///
    /// # Returns
    ///
    /// On success, a ProcessIdentifier with the given value.
    /// On failure, an error if the value exceeds i32::MAX.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub fn try_from_u32(raw: u32) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& (raw as int) <= (i32::MAX as int)
                &&& result->Ok_0.spec_value() == raw as int
                &&& result->Ok_0.spec_is_non_negative()
            },
            result is Err ==> (raw as int) > (i32::MAX as int),
    {
        if raw > i32::MAX as u32 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(ProcessIdentifier { value: raw as i32 })
    }

    /// Creates a ProcessIdentifier from a u64 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw u64 value.
    ///
    /// # Returns
    ///
    /// On success, a ProcessIdentifier with the given value.
    /// On failure, an error if the value exceeds i32::MAX.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub fn try_from_u64(raw: u64) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& (raw as int) <= (i32::MAX as int)
                &&& result->Ok_0.spec_value() == raw as int
                &&& result->Ok_0.spec_is_non_negative()
            },
            result is Err ==> (raw as int) > (i32::MAX as int),
    {
        if raw > i32::MAX as u64 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid process identifier"));
        }
        Ok(ProcessIdentifier { value: raw as i32 })
    }

    /// Converts the ProcessIdentifier to native-endian bytes.
    ///
    /// # Returns
    ///
    /// A 4-byte array in native-endian order.
    ///
    /// # Note on Verification
    ///
    /// This function uses `external_body` because Verus cannot reason about
    /// byte-level integer representation. The round-trip property is assumed
    /// based on Rust's i32::to_ne_bytes/from_ne_bytes semantics.
    /// See `axiom_byte_roundtrip` proof lemma for the assumed property.
    #[verifier::external_body]
    pub fn to_ne_bytes(&self) -> (result: [u8; 4])
        ensures
            result == self.spec_to_ne_bytes(),
    {
        self.value.to_ne_bytes()
    }

    /// Creates a ProcessIdentifier from native-endian bytes.
    ///
    /// # Parameters
    ///
    /// - `bytes`: A 4-byte array in native-endian order.
    ///
    /// # Returns
    ///
    /// A ProcessIdentifier with the value decoded from the bytes.
    ///
    /// # Note on Verification
    ///
    /// This function uses `external_body` because Verus cannot reason about
    /// byte-level integer representation. The round-trip property is assumed
    /// based on Rust's i32::from_ne_bytes semantics.
    /// See `axiom_byte_roundtrip` proof lemma for the assumed property.
    #[verifier::external_body]
    pub fn from_ne_bytes(bytes: [u8; 4]) -> (result: ProcessIdentifier)
        ensures
            result.spec_value() == Self::spec_from_ne_bytes(bytes),
    {
        ProcessIdentifier { value: i32::from_ne_bytes(bytes) }
    }

    /// Checks if two ProcessIdentifiers are equal.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if both have the same value.
    #[inline]
    pub fn eq(&self, other: &ProcessIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() == other.spec_value()),
    {
        self.value == other.value
    }

    /// Compares two ProcessIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is less than other.
    #[inline]
    pub fn lt(&self, other: &ProcessIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() < other.spec_value()),
    {
        self.value < other.value
    }

    /// Compares two ProcessIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is less than or equal to other.
    #[inline]
    pub fn le(&self, other: &ProcessIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() <= other.spec_value()),
    {
        self.value <= other.value
    }

    /// Creates a default ProcessIdentifier (value 0).
    ///
    /// # Returns
    ///
    /// A ProcessIdentifier with value 0.
    #[inline]
    pub fn default_value() -> (result: ProcessIdentifier)
        ensures
            result.spec_value() == 0,
            result.spec_is_kernel(),
    {
        ProcessIdentifier { value: 0 }
    }
}

} // verus!
