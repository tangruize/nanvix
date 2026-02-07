// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ThreadIdentifier Implementation
//!
//! A type that represents a thread identifier (TID).
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
include!("tid.spec.rs");

// Include proofs.
include!("tid.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A type that represents a thread identifier.
///
/// # Description
///
/// ThreadIdentifier wraps an i32 value that uniquely identifies a thread.
/// Special constants:
/// - KERNEL (0): The kernel thread.
/// - INITD (1): The init daemon thread.
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
pub struct ThreadIdentifier {
    /// The raw i32 value of the thread identifier.
    /// Note: pub for Verus spec access; prefer using accessor methods.
    pub value: i32,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ThreadIdentifier {
    /// Raw identifier for the kernel thread.
    pub const KERNEL_RAW: i32 = 0;

    /// Error message for invalid thread identifier conversions.
    pub(crate) const PARSE_ERROR_MESSAGE: &'static str = "invalid thread identifier";

    /// Identifier of the kernel thread.
    pub const KERNEL: ThreadIdentifier = ThreadIdentifier { value: Self::KERNEL_RAW };

    /// Identifier of the init daemon thread.
    pub const INITD: ThreadIdentifier = ThreadIdentifier { value: 1 };

    /// Creates a new ThreadIdentifier from an i32 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw i32 value.
    ///
    /// # Returns
    ///
    /// A new ThreadIdentifier with the given value.
    #[inline]
    pub(crate) fn from_i32(raw: i32) -> (result: ThreadIdentifier)
        ensures
            result.spec_value() == raw as int,
            result@ == (ThreadIdentifierView { value: raw as int }),
    {
        ThreadIdentifier { value: raw }
    }

    /// Converts the ThreadIdentifier to an i32 value.
    ///
    /// # Returns
    ///
    /// The raw i32 value.
    #[inline]
    pub(crate) fn into_i32(self) -> (result: i32)
        ensures
            result as int == self.spec_value(),
    {
        self.value
    }

    /// Converts the ThreadIdentifier to an isize value.
    ///
    /// # Returns
    ///
    /// The value as isize.
    #[inline]
    pub(crate) fn into_isize(self) -> (result: isize)
        ensures
            result as int == self.spec_value(),
    {
        self.value as isize
    }

    /// Converts the ThreadIdentifier to an i64 value.
    ///
    /// # Returns
    ///
    /// The value as i64.
    #[inline]
    pub(crate) fn into_i64(self) -> (result: i64)
        ensures
            result as int == self.spec_value(),
    {
        self.value as i64
    }

    /// Tries to convert the ThreadIdentifier to a usize value.
    ///
    /// # Returns
    ///
    /// On success, the value as usize. On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub(crate) fn try_into_usize(self) -> (result: Result<usize, Error>)
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

    /// Tries to convert the ThreadIdentifier to a u32 value.
    ///
    /// # Returns
    ///
    /// On success, the value as u32. On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub(crate) fn try_into_u32(self) -> (result: Result<u32, Error>)
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

    /// Tries to convert the ThreadIdentifier to a u64 value.
    ///
    /// # Returns
    ///
    /// On success, the value as u64. On failure, an error indicating invalid argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub(crate) fn try_into_u64(self) -> (result: Result<u64, Error>)
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

    /// Creates a ThreadIdentifier from an isize value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw isize value.
    ///
    /// # Returns
    ///
    /// On success, a ThreadIdentifier with the given value.
    /// On failure, an error if the value is out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is outside the i32 range.
    pub(crate) fn try_from_isize(raw: isize) -> (result: Result<ThreadIdentifier, Error>)
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
        Ok(ThreadIdentifier { value: raw as i32 })
    }

    /// Creates a ThreadIdentifier from an i64 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw i64 value.
    ///
    /// # Returns
    ///
    /// On success, a ThreadIdentifier with the given value.
    /// On failure, an error if the value is out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is outside the i32 range.
    pub(crate) fn try_from_i64(raw: i64) -> (result: Result<ThreadIdentifier, Error>)
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
        Ok(ThreadIdentifier { value: raw as i32 })
    }

    /// Creates a ThreadIdentifier from a usize value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw usize value.
    ///
    /// # Returns
    ///
    /// On success, a ThreadIdentifier with the given value.
    /// On failure, an error if the value exceeds i32::MAX.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub(crate) fn try_from_usize(raw: usize) -> (result: Result<ThreadIdentifier, Error>)
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
        Ok(ThreadIdentifier { value: raw as i32 })
    }

    /// Creates a ThreadIdentifier from a u32 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw u32 value.
    ///
    /// # Returns
    ///
    /// On success, a ThreadIdentifier with the given value.
    /// On failure, an error if the value exceeds i32::MAX.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub(crate) fn try_from_u32(raw: u32) -> (result: Result<ThreadIdentifier, Error>)
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
        Ok(ThreadIdentifier { value: raw as i32 })
    }

    /// Creates a ThreadIdentifier from a u64 value.
    ///
    /// # Parameters
    ///
    /// - `raw`: The raw u64 value.
    ///
    /// # Returns
    ///
    /// On success, a ThreadIdentifier with the given value.
    /// On failure, an error if the value exceeds i32::MAX.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub(crate) fn try_from_u64(raw: u64) -> (result: Result<ThreadIdentifier, Error>)
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
        Ok(ThreadIdentifier { value: raw as i32 })
    }

    /// Converts the ThreadIdentifier to native-endian bytes.
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

    /// Creates a ThreadIdentifier from native-endian bytes.
    ///
    /// # Parameters
    ///
    /// - `bytes`: A 4-byte array in native-endian order.
    ///
    /// # Returns
    ///
    /// A ThreadIdentifier with the value decoded from the bytes.
    ///
    /// # Note on Verification
    ///
    /// This function uses `external_body` because Verus cannot reason about
    /// byte-level integer representation. The round-trip property is assumed
    /// based on Rust's i32::from_ne_bytes semantics.
    /// See `axiom_byte_roundtrip` proof lemma for the assumed property.
    #[verifier::external_body]
    pub fn from_ne_bytes(bytes: [u8; 4]) -> (result: ThreadIdentifier)
        ensures
            result.spec_value() == Self::spec_from_ne_bytes(bytes),
    {
        ThreadIdentifier { value: i32::from_ne_bytes(bytes) }
    }

    /// Checks if two ThreadIdentifiers are equal.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if both have the same value.
    #[inline]
    pub(crate) fn eq(&self, other: &ThreadIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() == other.spec_value()),
    {
        self.value == other.value
    }

    /// Checks if two ThreadIdentifiers are not equal.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if values differ.
    #[inline]
    pub(crate) fn ne(&self, other: &ThreadIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() != other.spec_value()),
    {
        self.value != other.value
    }

    /// Compares two ThreadIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is less than other.
    #[inline]
    pub(crate) fn lt(&self, other: &ThreadIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() < other.spec_value()),
    {
        self.value < other.value
    }

    /// Compares two ThreadIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is less than or equal to other.
    #[inline]
    pub(crate) fn le(&self, other: &ThreadIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() <= other.spec_value()),
    {
        self.value <= other.value
    }

    /// Compares two ThreadIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is greater than other.
    #[inline]
    pub(crate) fn gt(&self, other: &ThreadIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() > other.spec_value()),
    {
        self.value > other.value
    }

    /// Compares two ThreadIdentifiers for ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if self is greater than or equal to other.
    #[inline]
    pub(crate) fn ge(&self, other: &ThreadIdentifier) -> (result: bool)
        ensures
            result == (self.spec_value() >= other.spec_value()),
    {
        self.value >= other.value
    }

    /// Compares two ThreadIdentifiers, returning an Ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ThreadIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// The ordering relationship between self and other.
    pub(crate) fn cmp_ord(&self, other: &ThreadIdentifier) -> (result: core::cmp::Ordering)
        ensures
            result == self.spec_cmp(other),
    {
        if self.value < other.value {
            core::cmp::Ordering::Less
        } else if self.value > other.value {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }

}

} // verus!

//==================================================================================================
// External Trait Implementations
//==================================================================================================

// These trait implementations wrap the verified methods to provide the standard Rust API.
// They are marked external because Verus cannot verify trait implementations directly.

impl PartialEq for ThreadIdentifier {
    /// Compares two ThreadIdentifiers for equality.
    fn eq(&self, other: &Self) -> bool {
        Self::eq(self, other)
    }
}

impl Eq for ThreadIdentifier {}

impl PartialOrd for ThreadIdentifier {
    /// Compares two ThreadIdentifiers for ordering.
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ThreadIdentifier {
    /// Compares two ThreadIdentifiers for total ordering.
    /// Delegates to verified `cmp_ord` method.
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cmp_ord(other)
    }
}

impl core::fmt::Debug for ThreadIdentifier {
    /// Formats the ThreadIdentifier for debugging.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl From<i32> for ThreadIdentifier {
    /// Creates a ThreadIdentifier from an i32 value.
    fn from(raw: i32) -> Self {
        Self::from_i32(raw)
    }
}

impl From<ThreadIdentifier> for i32 {
    /// Converts a ThreadIdentifier to an i32 value.
    fn from(tid: ThreadIdentifier) -> i32 {
        tid.into_i32()
    }
}

impl From<ThreadIdentifier> for isize {
    /// Converts a ThreadIdentifier to an isize value.
    fn from(tid: ThreadIdentifier) -> isize {
        tid.into_isize()
    }
}

impl From<ThreadIdentifier> for i64 {
    /// Converts a ThreadIdentifier to an i64 value.
    fn from(tid: ThreadIdentifier) -> i64 {
        tid.into_i64()
    }
}

impl TryFrom<isize> for ThreadIdentifier {
    type Error = Error;

    /// Creates a ThreadIdentifier from an isize value.
    fn try_from(raw: isize) -> Result<Self, Self::Error> {
        Self::try_from_isize(raw)
    }
}

impl TryFrom<i64> for ThreadIdentifier {
    type Error = Error;

    /// Creates a ThreadIdentifier from an i64 value.
    fn try_from(raw: i64) -> Result<Self, Self::Error> {
        Self::try_from_i64(raw)
    }
}

impl TryFrom<usize> for ThreadIdentifier {
    type Error = Error;

    /// Creates a ThreadIdentifier from a usize value.
    fn try_from(raw: usize) -> Result<Self, Self::Error> {
        Self::try_from_usize(raw)
    }
}

impl TryFrom<u32> for ThreadIdentifier {
    type Error = Error;

    /// Creates a ThreadIdentifier from a u32 value.
    fn try_from(raw: u32) -> Result<Self, Self::Error> {
        Self::try_from_u32(raw)
    }
}

impl TryFrom<u64> for ThreadIdentifier {
    type Error = Error;

    /// Creates a ThreadIdentifier from a u64 value.
    fn try_from(raw: u64) -> Result<Self, Self::Error> {
        Self::try_from_u64(raw)
    }
}

impl TryFrom<ThreadIdentifier> for usize {
    type Error = Error;

    /// Converts a ThreadIdentifier to a usize value.
    fn try_from(tid: ThreadIdentifier) -> Result<Self, Self::Error> {
        tid.try_into_usize()
    }
}

impl TryFrom<ThreadIdentifier> for u32 {
    type Error = Error;

    /// Converts a ThreadIdentifier to a u32 value.
    fn try_from(tid: ThreadIdentifier) -> Result<Self, Self::Error> {
        tid.try_into_u32()
    }
}

impl TryFrom<ThreadIdentifier> for u64 {
    type Error = Error;

    /// Converts a ThreadIdentifier to a u64 value.
    fn try_from(tid: ThreadIdentifier) -> Result<Self, Self::Error> {
        tid.try_into_u64()
    }
}
