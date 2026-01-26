// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Unified Verus Verification Crate
//!
//! This crate contains verified kernel allocator components.
//! Each module builds on lower-level verified modules.
//!
//! ## Module Hierarchy
//!
//! - `error` - Error types
//! - `raw_array` - Raw memory array (uses error)
//! - `bitmap` - Bitmap allocator (uses error, raw_array)
//! - `slab` - Slab allocator (uses error, raw_array, bitmap)
//! - `kheap` - Kernel heap allocator (uses error, slab)
//! - `frame_address` - Frame address types
//! - `frame` - Frame allocator (uses error, bitmap, frame_address)
//! - `upool` - User frame pool (uses error, frame, frame_address)
//! - `kpool` - Kernel frame pool (uses error, frame, frame_address)

pub mod error;
pub mod raw_array;
pub mod bitmap;
pub mod slab;
pub mod kheap;
pub mod frame_address;
pub mod frame;
pub mod upool;
pub mod kpool;
