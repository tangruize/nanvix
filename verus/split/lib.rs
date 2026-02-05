// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Nanvix Verus Verification (Split Organization)
//!
//! This crate provides formally verified specifications and proofs for Nanvix components.
//!
//! ## Organization
//!
//! Each module is split into three files for better readability:
//! - `lib.rs` - Implementation code with requires/ensures annotations
//! - `lib.spec.rs` - Specification functions, invariants, and View traits
//! - `lib.proof.rs` - Lemmas and proof functions
//!
//! The implementation code only calls lemmas from the proof file; actual proofs
//! are not inlined in the code.

pub mod libs;
pub mod kernel;
