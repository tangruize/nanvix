// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Verus Macro Stub
//!
//! This crate provides stub proc-macros that allow Verus verification annotations
//! to be used in normal Rust builds. When not verifying with Verus, these macros
//! simply pass through the annotated items unchanged.
//!
//! ## Usage
//!
//! Use `verus_stub` crate instead of importing this crate directly.

extern crate proc_macro;
use proc_macro::TokenStream;

/// Marks an item as Verus-aware.
///
/// When verifying with Verus, this enables verification for the item.
/// In normal builds, this is a no-op.
#[proc_macro_attribute]
pub fn verus_verify(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Specifies pre/postconditions for executable code.
///
/// When verifying with Verus, this adds requires/ensures clauses.
/// In normal builds, this is a no-op.
#[proc_macro_attribute]
pub fn verus_spec(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Wraps code that should be parsed by Verus.
///
/// When verifying with Verus, the code inside is verified.
/// In normal builds, the code is passed through unchanged.
#[proc_macro]
pub fn verus(input: TokenStream) -> TokenStream {
    input
}

/// Inserts proof code.
///
/// When verifying with Verus, the proof code helps the solver.
/// In normal builds, this produces nothing.
#[proc_macro]
pub fn proof(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// Declares proof variables.
///
/// When verifying with Verus, declares ghost variables for proofs.
/// In normal builds, this produces nothing.
#[proc_macro]
pub fn proof_decl(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// Proof with tracked resources.
///
/// When verifying with Verus, handles tracked resources in proofs.
/// In normal builds, this produces nothing.
#[proc_macro]
pub fn proof_with(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
