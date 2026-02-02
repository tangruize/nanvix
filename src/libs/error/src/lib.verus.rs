// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Error Verification Specifications
//
// This file provides Verus verification support for Error and ErrorCode types.

use vstd::prelude::*;

verus! {

//==================================================================================================
// ErrorCode Specification
//==================================================================================================

/// Spec function to get the underlying i32 value of an ErrorCode.
pub open spec fn error_code_value(code: ErrorCode) -> i32 {
    match code {
        ErrorCode::InvalidArgument => 22,
        ErrorCode::OutOfMemory => 12,
        ErrorCode::ResourceBusy => 16,
        ErrorCode::BadAddress => 14,
        _ => 0,  // Default for other variants (not fully specified).
    }
}

//==================================================================================================
// Error Specification
//==================================================================================================

impl Error {
    /// Spec function to get the error code.
    pub open spec fn spec_code(&self) -> ErrorCode {
        self.code
    }

    /// Spec function to get the error reason.
    pub open spec fn spec_reason(&self) -> &'static str {
        self.reason
    }
}

} // verus!
