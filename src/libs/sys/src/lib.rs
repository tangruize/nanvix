// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Configuration
//==================================================================================================

#![deny(clippy::all)]
#![forbid(clippy::large_stack_frames)]
#![forbid(clippy::large_stack_arrays)]
#![cfg_attr(not(feature = "verus"), feature(never_type))]
#![cfg_attr(not(feature = "std"), no_std)]

//==================================================================================================
// Types
//==================================================================================================

/// Never type: a type that can never be constructed.
/// Uses `!` on nightly Rust, `Infallible` when compiling with Verus (stable).
#[cfg(not(feature = "verus"))]
pub type Never = !;

/// Never type: a type that can never be constructed.
/// Uses `Infallible` when compiling with Verus (stable).
#[cfg(feature = "verus")]
pub type Never = ::core::convert::Infallible;

//==================================================================================================
// Modules
//==================================================================================================

// Exit status.
mod exit_status;

/// System configuration constants.
mod sys;

//==================================================================================================
// Exports
//==================================================================================================

pub use exit_status::*;
pub use sys::*;
