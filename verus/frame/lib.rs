// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Verified Frame Allocator
//!
//! This crate provides a Verus-verified version of the Nanvix frame allocator.
//! The frame allocator manages physical memory frames using a bitmap to track
//! allocation status.
//!
//! ## Modules
//!
//! - `error`: Error types for the allocator.
//! - `bitmap`: Bitmap allocator for tracking frame allocation (trusted).
//! - `frame_address`: Frame address and frame number types (trusted).
//! - `frame_core`: The main frame allocator implementation (verified).
//!
//! ## Verification
//!
//! ```bash
//! cd verus/frame
//! verus --crate-type lib lib.rs
//! ```
//!
//! ## Key Properties Verified
//!
//! 1. **Memory Safety**: All allocated frames have disjoint memory regions.
//! 2. **Bounds Checking**: All frame indices are within valid range.
//! 3. **Allocation Validity**: Allocated addresses are correctly computed.
//! 4. **Deallocation Correctness**: Freed frames become available.
//! 5. **Invariant Preservation**: Allocator invariants are maintained across operations.
//! 6. **Liveness**: If free frames exist, allocation succeeds.
//! 7. **Ghost Ownership**: Tracked permissions prove exclusive frame access.
//!
//! ## API Safety Notes
//!
//! **Tracked vs Untracked Allocation:**
//! - `alloc_tracked` / `free_tracked`: Return/require `FramePermission` for exclusive ownership
//! - `alloc` / `free_untracked`: No permission tracking, use for legacy/unverified code
//!
//! **Warning:** Mixing tracked and untracked APIs can break ownership guarantees.
//! Use `free_tracked` for frames allocated via `alloc_tracked`.
//!
//! ## Architecture
//!
//! Target: x86 (32-bit). `MAX_FRAME_NUMBER` is calculated for a 32-bit address space.

#![allow(dead_code)]

pub mod error;
pub mod bitmap;
pub mod frame_address;
pub mod permission;
pub mod frame_core;

// Re-export main types.
pub use error::{Error, ErrorCode};
pub use bitmap::Bitmap;
pub use frame_address::{FrameAddress, FrameNumber, PageAlignedPhysAddr, FRAME_SIZE, MAX_FRAME_NUMBER};
pub use permission::{FramePermission, FramePermissionSet, RangePermission};
pub use frame_core::{FrameAllocator, FrameAllocatorView};
