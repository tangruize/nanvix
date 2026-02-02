// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Verus Stub
//!
//! This crate provides stub macros for Verus verification annotations that allow
//! verified code to compile without Verus. When building normally (without Verus),
//! all Verus annotations become no-ops.
//!
//! ## Usage
//!
//! Add `use verus_stub::*;` at the top of modules that use Verus annotations.
//!
//! ## Features
//!
//! - `default`: Uses stub macros (for normal Rust builds)
//! - `disable`: Disables stub macros and enables real Verus verification
//!
//! ## Pattern
//!
//! This follows the SVSM project's pattern for integrating Verus verification
//! with normal Rust builds through conditional compilation.

#![no_std]

// When verifying with Verus (feature "disable" enabled), use the real Verus macros.
#[cfg(feature = "disable")]
pub use verus_builtin_macros::*;

#[cfg(feature = "disable")]
pub use vstd::prelude::*;

// For normal builds (default), use the stub macros.
#[cfg(not(feature = "disable"))]
pub use verus_macro_stub::*;
