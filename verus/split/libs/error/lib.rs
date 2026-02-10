// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Error Handling - Implementation

use vstd::prelude::*;

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// Error code for various adverse conditions.
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
    /// No such entry.
    NoSuchEntry = 2,
    /// No such process.
    NoSuchProcess = 3,
}

/// An error type that combines an error code with a reason string.
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: &'static str,
}

//==================================================================================================
// Implementation
//==================================================================================================

impl ErrorCode {
    /// Returns the error code as an `i32`.
    pub fn get(&self) -> i32 {
        *self as i32
    }
}

impl Error {
    /// Creates a new error.
    pub fn new(code: ErrorCode, reason: &'static str) -> (result: Self)
        ensures
            result.code == code,
            result.reason == reason,
    {
        Self { code, reason }
    }

    /// Logs this error (external_body - no verification effect).
    #[verifier::external_body]
    pub fn log(&self) {
        // In actual implementation: error!("{:?}", self);
    }
}

/// Logs an error with additional context.
#[verifier::external_body]
pub fn log_error_with_context(error: &Error, context: &str) {
    let _ = (error, context);
}

#[verifier::external]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.reason)
    }
}

} // verus!
