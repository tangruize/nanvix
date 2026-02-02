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
//! - `verus`: Would enable actual Verus verification (not yet implemented)
//!
//! ## Pattern
//!
//! This follows the SVSM project's pattern for integrating Verus verification
//! with normal Rust builds through conditional compilation.

#![no_std]
#![allow(unexpected_cfgs)]

// When verifying with Verus, we would use the real Verus macros.
// #[cfg(feature = "verus")]
// pub use verus_builtin_macros::*;
// #[cfg(feature = "verus")]
// pub use vstd::prelude::*;

// For normal builds, use the stub macros.
pub use verus_macro_stub::*;
