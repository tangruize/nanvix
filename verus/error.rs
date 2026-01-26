// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Error Handling
//==================================================================================================

use vstd::prelude::*;

verus! {

///
/// # Description
///
/// Error code for various adverse conditions.
///
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum ErrorCode {
    /// Invalid argument.
    InvalidArgument = 22,
    /// Out of memory.
    OutOfMemory = 12,
    /// Device or resource busy.
    ResourceBusy = 16,
    /// Bad address.
    BadAddress = 14,
}

impl ErrorCode {
    ///
    /// # Description
    ///
    /// Returns the error code as an `i32`.
    ///
    pub fn get(&self) -> i32 {
        *self as i32
    }
}

///
/// # Description
///
/// An error type that combines an error code with a reason string.
///
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: &'static str,
}

impl Error {
    ///
    /// # Description
    ///
    /// Creates a new error.
    ///
    /// # Parameters
    ///
    /// - `code`: The error code.
    /// - `reason`: A static string describing the error reason.
    ///
    /// # Returns
    ///
    /// A new error instance.
    ///
    pub fn new(code: ErrorCode, reason: &'static str) -> Self {
        Self { code, reason }
    }
}

} // verus!

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.reason)
    }
}
