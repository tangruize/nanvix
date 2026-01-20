// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Verified Kernel Heap Allocator
//!
//! This crate provides a Verus-verified version of the Nanvix kernel heap allocator.
//! The Kheap manages multiple Slab allocators of different block sizes to efficiently
//! handle allocation requests of various sizes.
//!
//! ## Modules
//!
//! - `error`: Error types for the allocator.
//! - `slab`: Slab allocator abstraction (trusted, verified separately in ../slab/).
//! - `kheap_core`: The main kernel heap implementation (verified).
//!
//! ## Verification
//!
//! ```bash
//! cd verus/kheap
//! verus --crate-type lib lib.rs
//! ```
//!
//! ## Key Properties Verified
//!
//! 1. **Correct Slab Selection**: Each allocation size maps to the appropriate slab.
//! 2. **No Overlap Between Slabs**: Each slab manages a disjoint memory region.
//! 3. **Allocation Validity**: Allocated addresses are within the correct slab's range.
//! 4. **Deallocation Correctness**: Deallocations target the correct slab.
//! 5. **Invariant Preservation**: The heap invariant is maintained across operations.

#![allow(dead_code)]

pub mod error;
pub mod slab;
pub mod kheap_core;

// Re-export main types.
pub use error::{Error, ErrorCode};
pub use slab::{Slab, SlabView};
pub use kheap_core::{
    Kheap, KheapView,
    SlabSize,
    layout_to_slab_size,
    spec_layout_to_slab_size,
    NUM_OF_SLABS,
    MIN_SLAB_SIZE,
    MIN_HEAP_SIZE,
    PAGE_SIZE,
};
