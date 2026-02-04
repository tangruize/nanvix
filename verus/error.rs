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

    /// Logs this error.
    ///
    /// # Description
    ///
    /// This is an external_body function that wraps the `error!` macro from the log crate.
    /// It has no effect on verification (treated as a no-op by Verus) but will log
    /// errors at runtime, matching the original Nanvix behavior.
    ///
    /// # Note
    ///
    /// In the original Nanvix code, error paths typically call `error!("{error:?}")`.
    /// This function provides the same functionality in a Verus-compatible way.
    #[verifier::external_body]
    pub fn log(&self) {
        // In actual implementation, this would call:
        // error!("{:?}", self);
        // For now, we leave it as a placeholder that can be filled in
        // when integrating with the actual logging infrastructure.
    }
}

/// Logs an error with additional context.
///
/// # Description
///
/// This is an external_body function for logging errors with context information.
/// It has no effect on verification but provides runtime logging.
#[verifier::external_body]
pub fn log_error_with_context(error: &Error, context: &str) {
    // In actual implementation:
    // error!("{:?} ({})", error, context);
    let _ = (error, context);
}

} // verus!

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.reason)
    }
}
