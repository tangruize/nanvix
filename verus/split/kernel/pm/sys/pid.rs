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
//! - Error paths return `ErrorCode::InvalidArgument`.
//! - Byte serialization round-trips correctly.
//! - Layout: size == 4 bytes, alignment == 4 bytes (matching original ABI).
//!
//! ## Trust Boundary: Byte Serialization
//!
//! The `to_ne_bytes`/`from_ne_bytes` functions and their corresponding axioms
//! (`axiom_byte_roundtrip`, `axiom_decode_encode_roundtrip`) are an explicit
//! trust boundary. They use `external_body` with uninterpreted spec functions
//! because Verus cannot reason about byte-level integer representation.
//! These axioms are justified by Rust's `i32::to_ne_bytes`/`i32::from_ne_bytes`
//! semantics which guarantee round-trip fidelity. Consumers of this module
//! should be aware that these two axioms are assumed, not proven.
//!
//! ## Trust Boundary: Layout Assertions
//!
//! The `lemma_size_eq_i32` and `lemma_align_eq_i32` proof lemmas use
//! `external_body` because Verus cannot reason about `core::mem::size_of`
//! and `core::mem::align_of`. These are justified by the `#[repr(C)]`
//! attribute on a single-field struct wrapping `i32`.

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
/// The `value` field is `pub` for Verus spec reasoning. The original type
/// uses a tuple struct with private field. Verified code should use accessor methods
/// (`into_i32`, `from_i32`) rather than direct field access to maintain encapsulation.
///
/// # Representation
///
/// Uses `#[repr(C)]` to match the original type's FFI-compatible layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessIdentifier {
    /// The raw i32 value of the process identifier.
    /// Note: pub for Verus spec access; prefer using accessor methods.
    pub value: i32,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessIdentifier {
    /// Raw identifier for the kernel process.
    pub const KERNEL_RAW: i32 = 0;

    /// Error message for invalid process identifier conversions.
    const PARSE_ERROR_MESSAGE: &'static str = "invalid process identifier";

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
            result is Err ==> {
                &&& !self.spec_is_non_negative()
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
            result is Err ==> {
                &&& !self.spec_is_non_negative()
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
            result is Err ==> {
                &&& !self.spec_is_non_negative()
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if self.value < 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
                &&& Self::spec_in_i32_range(raw as int)
                &&& result->Ok_0.spec_value() == raw as int
            },
            result is Err ==> {
                &&& !Self::spec_in_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw < i32::MIN as isize || raw > i32::MAX as isize {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
                &&& Self::spec_in_i32_range(raw as int)
                &&& result->Ok_0.spec_value() == raw as int
            },
            result is Err ==> {
                &&& !Self::spec_in_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw < i32::MIN as i64 || raw > i32::MAX as i64 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
                &&& Self::spec_in_non_negative_i32_range(raw as int)
                &&& result->Ok_0.spec_value() == raw as int
                &&& result->Ok_0.spec_is_non_negative()
            },
            result is Err ==> {
                &&& !Self::spec_in_non_negative_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw > i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
                &&& Self::spec_in_non_negative_i32_range(raw as int)
                &&& result->Ok_0.spec_value() == raw as int
                &&& result->Ok_0.spec_is_non_negative()
            },
            result is Err ==> {
                &&& !Self::spec_in_non_negative_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw > i32::MAX as u32 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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
                &&& Self::spec_in_non_negative_i32_range(raw as int)
                &&& result->Ok_0.spec_value() == raw as int
                &&& result->Ok_0.spec_is_non_negative()
            },
            result is Err ==> {
                &&& !Self::spec_in_non_negative_i32_range(raw as int)
                &&& result->Err_0.code == ErrorCode::InvalidArgument
                &&& result->Err_0.reason == Self::PARSE_ERROR_MESSAGE
            },
    {
        if raw > i32::MAX as u64 {
            return Err(Error::new(ErrorCode::InvalidArgument, Self::PARSE_ERROR_MESSAGE));
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

    /// Compares two ProcessIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is greater than other.
    #[inline]
    pub fn gt(&self, other: &ProcessIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() > other.spec_value()),
    {
        self.value > other.value
    }

    /// Compares two ProcessIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is greater than or equal to other.
    #[inline]
    pub fn ge(&self, other: &ProcessIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() >= other.spec_value()),
    {
        self.value >= other.value
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

//==================================================================================================
// External Trait Implementations
//==================================================================================================

// These trait implementations wrap the verified methods to provide the standard Rust API.
// They are marked external because Verus cannot verify trait implementations directly.

impl Default for ProcessIdentifier {
    /// Returns the default ProcessIdentifier (KERNEL, value 0).
    fn default() -> Self {
        Self::default_value()
    }
}

impl PartialEq for ProcessIdentifier {
    /// Compares two ProcessIdentifiers for equality.
    fn eq(&self, other: &Self) -> bool {
        Self::eq(self, other)
    }
}

impl Eq for ProcessIdentifier {}

impl PartialOrd for ProcessIdentifier {
    /// Compares two ProcessIdentifiers for ordering.
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProcessIdentifier {
    /// Compares two ProcessIdentifiers for total ordering.
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl core::fmt::Debug for ProcessIdentifier {
    /// Formats the ProcessIdentifier for debugging.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl From<i32> for ProcessIdentifier {
    /// Creates a ProcessIdentifier from an i32 value.
    fn from(raw: i32) -> Self {
        Self::from_i32(raw)
    }
}

impl From<ProcessIdentifier> for i32 {
    /// Converts a ProcessIdentifier to an i32 value.
    fn from(pid: ProcessIdentifier) -> i32 {
        pid.into_i32()
    }
}

impl From<ProcessIdentifier> for isize {
    /// Converts a ProcessIdentifier to an isize value.
    fn from(pid: ProcessIdentifier) -> isize {
        pid.into_isize()
    }
}

impl From<ProcessIdentifier> for i64 {
    /// Converts a ProcessIdentifier to an i64 value.
    fn from(pid: ProcessIdentifier) -> i64 {
        pid.into_i64()
    }
}

impl TryFrom<isize> for ProcessIdentifier {
    type Error = Error;

    /// Creates a ProcessIdentifier from an isize value.
    fn try_from(raw: isize) -> Result<Self, Self::Error> {
        Self::try_from_isize(raw)
    }
}

impl TryFrom<i64> for ProcessIdentifier {
    type Error = Error;

    /// Creates a ProcessIdentifier from an i64 value.
    fn try_from(raw: i64) -> Result<Self, Self::Error> {
        Self::try_from_i64(raw)
    }
}

impl TryFrom<usize> for ProcessIdentifier {
    type Error = Error;

    /// Creates a ProcessIdentifier from a usize value.
    fn try_from(raw: usize) -> Result<Self, Self::Error> {
        Self::try_from_usize(raw)
    }
}

impl TryFrom<u32> for ProcessIdentifier {
    type Error = Error;

    /// Creates a ProcessIdentifier from a u32 value.
    fn try_from(raw: u32) -> Result<Self, Self::Error> {
        Self::try_from_u32(raw)
    }
}

impl TryFrom<u64> for ProcessIdentifier {
    type Error = Error;

    /// Creates a ProcessIdentifier from a u64 value.
    fn try_from(raw: u64) -> Result<Self, Self::Error> {
        Self::try_from_u64(raw)
    }
}

impl TryFrom<ProcessIdentifier> for usize {
    type Error = Error;

    /// Converts a ProcessIdentifier to a usize value.
    fn try_from(pid: ProcessIdentifier) -> Result<Self, Self::Error> {
        pid.try_into_usize()
    }
}

impl TryFrom<ProcessIdentifier> for u32 {
    type Error = Error;

    /// Converts a ProcessIdentifier to a u32 value.
    fn try_from(pid: ProcessIdentifier) -> Result<Self, Self::Error> {
        pid.try_into_u32()
    }
}

impl TryFrom<ProcessIdentifier> for u64 {
    type Error = Error;

    /// Converts a ProcessIdentifier to a u64 value.
    fn try_from(pid: ProcessIdentifier) -> Result<Self, Self::Error> {
        pid.try_into_u64()
    }
}
