// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Slab Allocator
//!
//! This crate provides a verified slab allocator implementation.
//! The verification is done using Verus.
//!
//! ## Modules
//!
//! - `error`: Error types for the allocator.
//! - `raw_array`: Low-level array storage (trusted).
//! - `bitmap`: Bitmap allocator for tracking free blocks (trusted).
//! - `slab_core`: The main slab allocator implementation (verified).

#![allow(dead_code)]

pub mod error;
pub mod raw_array;
pub mod bitmap;
pub mod slab_core;

// Re-export main types.
pub use error::{Error, ErrorCode};
pub use bitmap::Bitmap;
pub use slab_core::{Slab, SlabView};
