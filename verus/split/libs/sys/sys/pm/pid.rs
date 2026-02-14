// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessIdentifier Implementation (libs)
//!
//! A type that represents a process identifier (PID).
//!
//! ## Verified Properties
//!
//! - Constants KERNEL and INITD have expected values (0 and 1).
//! - Conversion from/to i32 preserves value.
//! - Conversion to/from other integer types is safe when in range.
//! - Error paths return `ErrorCode::InvalidArgument`.
//! - Byte serialization round-trips correctly (trust boundary).
//! - Layout: size == 4 bytes, alignment == 4 bytes.
//!
//! ## Methodology Compliance
//!
//! - Step 1: `ProcessIdentifierView` uses `int` (not `i32`). `view()` is `closed`.
//! - Step 2: `inv()` is `pub closed spec fn`. Trivially true (any i32 is valid).
//! - Step 3: Public specs use `self@.value` (view), not `self.value` (impl).
//!           No `pub` spec functions on `impl ProcessIdentifier` beyond `inv()` and `view()`.
//! - Step 5: No `assume()`, `admit()`, or unjustified `external_body`.
//!
//! ## Trust Boundaries
//!
//! - Byte serialization: `to_ne_bytes`/`from_ne_bytes` use `external_body`
//!   (justified by Rust's i32 byte semantics).
//! - Layout assertions: `lemma_size_eq_i32`/`lemma_align_eq_i32` use `external_body`
//!   (justified by `#[repr(C)]` on a single-field i32 struct).

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
/// Special constants: KERNEL (0) and INITD (1).
///
/// # Representation
///
/// Uses `#[repr(C)]` for FFI-compatible layout. The `value` field is `pub`
/// for Verus spec reasoning; the original uses a private tuple field.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessIdentifier {
    /// The raw i32 value.
    pub value: i32,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessIdentifier {
    /// Raw identifier for the kernel process.
    pub const KERNEL_RAW: i32 = 0;

    /// Error message for invalid process identifier conversions.
    pub const PARSE_ERROR_MESSAGE: &'static str = "invalid process identifier";

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
            result@.value == raw as int,
            result@ == (ProcessIdentifierView { value: raw as int }),
            result.inv(),
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
        requires
            self.inv(),
        ensures
            result as int == self@.value,
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
        requires
            self.inv(),
        ensures
            result as int == self@.value,
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
        requires
            self.inv(),
        ensures
            result as int == self@.value,
    {
        self.value as i64
    }

    /// Tries to convert the ProcessIdentifier to a usize value.
    ///
    /// # Returns
    ///
    /// On success, the value as usize. On failure, an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub fn try_into_usize(self) -> (result: Result<usize, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& self@.is_non_negative()
                &&& result->Ok_0 as int == self@.value
            },
            result is Err ==> {
                &&& !self@.is_non_negative()
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
    /// On success, the value as u32. On failure, an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub fn try_into_u32(self) -> (result: Result<u32, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& self@.is_non_negative()
                &&& result->Ok_0 as int == self@.value
            },
            result is Err ==> {
                &&& !self@.is_non_negative()
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
    /// On success, the value as u64. On failure, an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative.
    pub fn try_into_u64(self) -> (result: Result<u64, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& self@.is_non_negative()
                &&& result->Ok_0 as int == self@.value
            },
            result is Err ==> {
                &&& !self@.is_non_negative()
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
    /// On success, a ProcessIdentifier. On failure, an error if out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is outside the i32 range.
    pub fn try_from_isize(raw: isize) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ProcessIdentifierView::in_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ProcessIdentifierView::in_i32_range(raw as int)
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
    /// On success, a ProcessIdentifier. On failure, an error if out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is outside the i32 range.
    pub fn try_from_i64(raw: i64) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ProcessIdentifierView::in_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ProcessIdentifierView::in_i32_range(raw as int)
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
    /// On success, a ProcessIdentifier. On failure, an error if out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub fn try_from_usize(raw: usize) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ProcessIdentifierView::in_non_negative_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0@.is_non_negative()
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ProcessIdentifierView::in_non_negative_i32_range(raw as int)
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
    /// On success, a ProcessIdentifier. On failure, an error if out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub fn try_from_u32(raw: u32) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ProcessIdentifierView::in_non_negative_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0@.is_non_negative()
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ProcessIdentifierView::in_non_negative_i32_range(raw as int)
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
    /// On success, a ProcessIdentifier. On failure, an error if out of i32 range.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds i32::MAX.
    pub fn try_from_u64(raw: u64) -> (result: Result<ProcessIdentifier, Error>)
        ensures
            result is Ok ==> {
                &&& ProcessIdentifierView::in_non_negative_i32_range(raw as int)
                &&& result->Ok_0@.value == raw as int
                &&& result->Ok_0@.is_non_negative()
                &&& result->Ok_0.inv()
            },
            result is Err ==> {
                &&& !ProcessIdentifierView::in_non_negative_i32_range(raw as int)
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
    /// Uses external_body because Verus cannot reason about byte-level
    /// integer representation. Justified by Rust's i32::to_ne_bytes semantics.
    #[verifier::external_body]
    pub fn to_ne_bytes(&self) -> (result: [u8; 4])
        requires
            self.inv(),
        ensures
            result == self@.to_ne_bytes_spec(),
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
    /// Uses external_body because Verus cannot reason about byte-level
    /// integer representation. Justified by Rust's i32::from_ne_bytes semantics.
    #[verifier::external_body]
    pub fn from_ne_bytes(bytes: [u8; 4]) -> (result: ProcessIdentifier)
        ensures
            result@.value == ProcessIdentifierView::from_ne_bytes_spec(bytes),
            result.inv(),
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
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value == other@.value),
    {
        self.value == other.value
    }

    /// Checks if two ProcessIdentifiers are not equal.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// True if values differ.
    #[inline]
    pub fn ne(&self, other: &ProcessIdentifier) -> (result: bool)
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value != other@.value),
    {
        self.value != other.value
    }

    /// Less than comparison.
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
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value < other@.value),
    {
        self.value < other.value
    }

    /// Less than or equal comparison.
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
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value <= other@.value),
    {
        self.value <= other.value
    }

    /// Greater than comparison.
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
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value > other@.value),
    {
        self.value > other.value
    }

    /// Greater than or equal comparison.
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
        requires
            self.inv(),
            other.inv(),
        ensures
            result == (self@.value >= other@.value),
    {
        self.value >= other.value
    }

    /// Compares two ProcessIdentifiers, returning an Ordering.
    ///
    /// # Parameters
    ///
    /// - `other`: The other ProcessIdentifier to compare with.
    ///
    /// # Returns
    ///
    /// The ordering relationship.
    pub fn cmp_ord(&self, other: &ProcessIdentifier) -> (result: core::cmp::Ordering)
        requires
            self.inv(),
            other.inv(),
        ensures
            (self@.value < other@.value) ==> result == core::cmp::Ordering::Less,
            (self@.value > other@.value) ==> result == core::cmp::Ordering::Greater,
            (self@.value == other@.value) ==> result == core::cmp::Ordering::Equal,
    {
        if self.value < other.value {
            core::cmp::Ordering::Less
        } else if self.value > other.value {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }

    /// Creates a default ProcessIdentifier (value 0).
    ///
    /// # Returns
    ///
    /// A ProcessIdentifier with value 0.
    #[inline]
    pub fn default_value() -> (result: ProcessIdentifier)
        ensures
            result@.value == 0,
            result@.is_kernel(),
            result.inv(),
    {
        ProcessIdentifier { value: 0 }
    }
}

} // verus!

//==================================================================================================
// External Trait Implementations
//==================================================================================================

// These trait implementations wrap the verified methods to provide the standard Rust API.
// They are outside verus! because Verus cannot verify trait implementations directly.

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
        self.cmp_ord(other)
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
