// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Kernel Frame Pool (Verified Implementation)
//!
//! This module provides a verified implementation of the Kernel Frame Pool (Kpool) which manages
//! kernel-space memory frames. The pool wraps a FrameAllocator and provides:
//!
//! - Single frame allocation via `alloc()`
//! - Contiguous range allocation with search via `alloc_contiguous(count)`
//! - Contiguous range booking via `alloc_range(start, count)`
//! - Non-contiguous multiple frame allocation via `alloc_noncontiguous()`
//! - Frame deallocation via `free()`, `free_range()`, `free_contiguous()`
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **No Double Allocation**: A frame can only be allocated if it's currently free.
//! 2. **No Double Free**: A frame can only be freed if it's currently allocated.
//! 3. **No Memory Aliasing**: All allocated frames have disjoint memory regions.
//! 4. **Valid Frame Indices**: All returned frames have valid indices within capacity.
//! 5. **Liveness**: Allocation succeeds when free frames exist; fails otherwise.
//! 6. **Contiguous Range Validity**: Range allocations return contiguous frame indices.
//! 7. **Count Correctness**: All operations maintain exact frame counts.
//! 8. **Provenance**: Frames carry pool_id and can only be freed to their originating pool.
//!
//! ## Provenance Tracking
//!
//! Each `KernelFrame` carries a `pool_id` that identifies which pool it was allocated from.
//! The `free()` function requires `kframe.spec_pool_id() == self@.id()`, ensuring frames
//! cannot be freed to the wrong pool. This prevents cross-pool aliasing bugs.
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `alloc()` | Allocate a single frame (with pool_id) |
//! | `alloc_contiguous(count)` | Search for and allocate contiguous range (original semantics) |
//! | `alloc_range(start, count)` | Book a specific contiguous range |
//! | `alloc_noncontiguous(count)` | Allocate multiple (possibly non-contiguous) frames |
//! | `free(kframe)` | Free a single frame (requires matching pool_id) |
//! | `free_range(start, count)` | Free a contiguous range |
//! | `free_contiguous(start, indices, count)` | Free with ghost index validation |
//!
//! ## Abstraction Decisions
//!
//! This verified implementation makes several intentional abstraction choices that differ
//! from the original `src/kernel/src/mm/phys/kpool.rs`:
//!
//! ### 1. Contiguous Allocation
//! - `alloc_contiguous(count)`: Searches for a free contiguous range (like original `alloc_many`)
//! - `alloc_range(start, count)`: Books a specific range (caller specifies start)
//!
//! ### 2. `alloc_noncontiguous()` vs Original `alloc_many()`
//! The original `alloc_many(count)` allocates a **contiguous** range internally.
//! Use `alloc_contiguous(count)` for original semantics.
//! `alloc_noncontiguous(count)` allocates individual frames that are **not necessarily contiguous**.
//!
//! ### 3. Return Types
//! - `alloc()` returns `Result<KernelFrame, Error>` (single frame with pool_id)
//! - `alloc_contiguous(count)` returns `Result<usize, Error>` (starting frame index)
//! - `alloc_range()` returns `Result<(), Error>` (range is booked by index)
//! - `alloc_noncontiguous()` returns `Result<Ghost<Seq<int>>, Error>` (ghost indices)
//!
//! ### 4. No `clear` Parameter
//! Clearing is orthogonal to allocation safety.
//!
//! ### 5. No RAII/Drop Semantics
//! Explicit `free()` calls make proof obligations clearer.
//!
//! ### 6. No `Deref`/`DerefMut` for Byte Access
//! Memory content modeling is beyond allocation safety verification.
//==================================================================================================

use crate::{
    libs::error::{
        Error,
        ErrorCode,
    },
    kernel::mm::phys::frame::{
        FrameAllocator,
        FrameAllocatorView,
    },
    kernel::hal::mem::types::address::frame::{
        FrameAddress,
        FrameNumber,
        FRAME_SIZE,
        MAX_FRAME_NUMBER,
    },
};
use vstd::prelude::*;

// Include specifications.
include!("kpool.spec.rs");

// Include proofs.
include!("kpool.proof.rs");


verus! {

//==================================================================================================

/// A type that represents a kernel frame.
///
/// KernelFrame is a wrapper around a FrameAddress that represents an allocated
/// frame from the kernel frame pool. The address is guaranteed to be page-aligned.
///
/// # Structural Equivalence to Original
///
/// The original `KernelFrame` has fields:
///   - `kpool: Rc<RefCell<KpoolInner>>` (shared pool reference for RAII Drop)
///   - `base: FrameAddress` (frame address)
///
/// This verified version replaces them with:
///   - `addr: FrameAddress` (renamed from `base`, semantically identical)
///   - `pool_id: usize` (replaces `Rc<RefCell<KpoolInner>>` for provenance tracking)
///
/// The `Rc<RefCell<KpoolInner>>` is replaced because Verus cannot model interior
/// mutability or reference-counted shared ownership. Instead, provenance is tracked
/// via `pool_id`, and deallocation is explicit (no Drop trait).
///
/// # Verus Limitations (Missing Trait Impls)
///
/// The following trait implementations from the original cannot be verified:
///   - `Deref for KernelFrame` (requires `unsafe { from_raw_parts(...) }`)
///   - `DerefMut for KernelFrame` (requires `unsafe { from_raw_parts_mut(...) }`)
///   - `Drop for KernelFrame` (requires `Rc<RefCell>` for RAII deallocation)
///   - `clear(&mut self)` (requires `DerefMut` for byte-level zeroing)
///
/// These are orthogonal to allocation safety: they deal with memory content
/// access and RAII lifecycle, not allocation/deallocation invariants.
pub struct KernelFrame {
    /// Frame address (page-aligned). Named `base` in the original.
    addr: FrameAddress,
    /// Pool identifier for provenance tracking.
    /// Replaces `Rc<RefCell<KpoolInner>>` from the original for pool association.
    pool_id: usize,
}

impl KernelFrame {
    //==============================================================================================

    /// Instantiates a kernel frame (internal use only).
    ///
    /// # Description
    ///
    /// This constructor is intended for internal use by Kpool. External code
    /// should not construct KernelFrame directly; frames should only be obtained
    /// through allocation APIs.
    ///
    /// # Parameters
    ///
    /// - `addr`: Frame address (must be page-aligned).
    /// - `pool_id`: Pool identifier for provenance tracking.
    ///
    /// # Returns
    ///
    /// A kernel frame wrapping the given address with provenance.
    fn new_internal(addr: FrameAddress, pool_id: usize) -> (result: KernelFrame)
        requires
            addr.spec_is_aligned(),
        ensures
            result.inv(),
            result@.frame_number == addr.spec_frame_number(),
            result@.pool_id == pool_id as int,
            // Backward-compatible accessors.
            result.spec_address() == addr,
            result.spec_is_aligned(),
            result.spec_frame_number() == addr.spec_frame_number(),
            result.spec_pool_id() == pool_id as int,
    {
        KernelFrame { addr, pool_id }
    }

    //==============================================================================================

    /// Returns the physical address of the target kernel frame.
    ///
    /// # Returns
    ///
    /// The frame address of the kernel frame.
    pub fn address(&self) -> (result: FrameAddress)
        ensures
            result == self.spec_address(),
            result.spec_frame_number() == self@.frame_number,
            result.spec_is_aligned() == self.spec_is_aligned(),
    {
        self.addr
    }


    /// Returns the pool identifier for provenance tracking.
    ///
    /// # Returns
    ///
    /// The pool identifier of the kernel frame.
    pub fn pool_id(&self) -> (result: usize)
        ensures
            result as int == self@.pool_id,
            result as int == self.spec_pool_id(),
    {
        self.pool_id
    }


    /// Returns the base address (same as `address()`, matching original `KernelFrame::base()`).
    ///
    /// # Equivalence to Original
    ///
    /// Original: `pub fn base(&self) -> FrameAddress { self.base }`
    /// Verified: `pub fn base(&self) -> FrameAddress { self.addr }`
    ///
    /// The field was renamed from `base` to `addr` in the verified struct, but
    /// the function semantics are identical: both return the frame's physical address.
    ///
    /// # Returns
    ///
    /// The base frame address of the kernel frame.
    pub fn base(&self) -> (result: FrameAddress)
        ensures
            result == self.spec_address(),
            result.spec_frame_number() == self@.frame_number,
            result.spec_is_aligned() == self.spec_is_aligned(),
    {
        self.addr
    }
}

//==================================================================================================

/// Abstract view of the kernel frame pool for specification purposes.
///
/// # Description
///
/// This view encapsulates the abstract state of the kernel frame pool.
/// Methods delegate to `allocator_view` for allocation state, while
/// `num_allocated_count` provides concrete counting for public method specs.
///
/// # Note on `allocator_view`
///
/// The `allocator_view` field remains public because Verus requires struct fields
/// referenced by `pub open spec fn` methods to be visible at the method's scope.
/// Making it private would prevent `pub open spec fn` bodies from compiling.
/// This is a known limitation of Verus's visibility model.
///
/// # Region and Provenance
///
/// The view includes:
/// - `base_addr`: The base physical address of the pool region
/// - `pool_id`: A unique identifier for provenance tracking
///
/// Frame addresses are computed as: `base_addr + frame_idx * FRAME_SIZE`.
/// The pool_id ensures frames are only freed to their originating pool.
#[verifier::ext_equal]
pub struct KpoolView {
    /// The underlying frame allocator view.
    /// Public due to Verus visibility constraints on `pub open spec fn`.
    pub allocator_view: FrameAllocatorView,
    /// Concrete count of allocated frames (from bitmap).
    /// Exposed via `num_allocated()` for public method specs.
    pub num_allocated_count: int,
    /// Base physical address of the pool region.
    /// Frame i has address: base_addr + i * FRAME_SIZE.
    pub base_addr: int,
    /// Unique pool identifier for provenance tracking.
    /// Prevents cross-pool aliasing bugs.
    pub pool_id: int,
}

//==================================================================================================

/// A structure that describes a pool of kernel frames.
///
/// The pool manages physical memory frames for kernel-space use.
/// It provides allocation and deallocation operations with memory safety guarantees.
///
/// # Structural Equivalence to Original
///
/// The original `Kpool` struct has:
///   - `inner: Rc<RefCell<KpoolInner>>` (where `KpoolInner` contains `region` and `bitmap`)
///
/// This verified version replaces it with:
///   - `frame_allocator: FrameAllocator` (absorbs `KpoolInner`'s `Bitmap` functionality)
///   - `pool_id: usize` (for provenance tracking, replacing `Rc<RefCell>` identity)
///
/// The `KpoolInner` struct is absent because its `TruncatedMemoryRegion` + `Bitmap`
/// are abstracted into `FrameAllocator`, and `Rc<RefCell>` cannot be modeled in Verus.
pub struct Kpool {
    /// Underlying frame allocator.
    frame_allocator: FrameAllocator,
    /// Pool identifier for provenance tracking.
    pool_id: usize,
}

impl Kpool {
    //==============================================================================================

    /// Instantiates a kernel frame pool from a frame allocator.
    ///
    /// # Equivalence to Original
    ///
    /// Original: `pub fn new(region: TruncatedMemoryRegion<PhysicalAddress>) -> Result<Self, Error>`
    /// Verified: `pub fn new(frame_allocator: FrameAllocator, pool_id: usize) -> Kpool`
    ///
    /// The original constructs a `Bitmap` from the region size and wraps it in
    /// `Rc<RefCell<KpoolInner>>`. This verified version accepts a pre-constructed
    /// `FrameAllocator` (which encapsulates the bitmap) because Verus cannot model
    /// `Rc<RefCell>` or `TruncatedMemoryRegion`. The `pool_id` parameter replaces
    /// the identity provided by `Rc` pointer equality.
    ///
    /// The allocation logic (bitmap-based frame tracking) is identical; only the
    /// construction interface differs due to Verus limitations.
    ///
    /// # Parameters
    ///
    /// - `frame_allocator`: Underlying frame allocator.
    /// - `pool_id`: Unique identifier for this pool (for provenance tracking).
    ///
    /// # Returns
    ///
    /// A kernel frame pool with the given pool_id.
    pub fn new(frame_allocator: FrameAllocator, pool_id: usize) -> (result: Kpool)
        requires
            frame_allocator.inv(),
        ensures
            result.inv(),
            result@.capacity() == frame_allocator@.capacity,
            result@.id() == pool_id as int,
            // Allocated set is preserved.
            forall|i: int| 0 <= i < result@.capacity() ==>
                result@.is_allocated(i) == frame_allocator@.is_allocated(i),
            // Allocation count is preserved from the frame allocator.
            result@.num_allocated() == frame_allocator.spec_num_allocated(),
            // Fresh initialization is preserved when count is also zero.
            (frame_allocator@.is_freshly_initialized() && frame_allocator.spec_num_allocated() == 0) ==>
                result@.is_freshly_initialized(),
    {
        Kpool { frame_allocator, pool_id }
    }


    /// Returns the capacity (number of frames managed).
    pub fn capacity(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.capacity()
    {
        self.frame_allocator.capacity()
    }


    /// Returns the pool identifier.
    pub fn get_pool_id(&self) -> (result: usize)
        ensures result as int == self@.id()
    {
        self.pool_id
    }

    //==============================================================================================

    /// Allocates a frame from the kernel frame pool.
    ///
    /// # Equivalence to Original
    ///
    /// Original: `pub fn alloc(&mut self, clear: bool) -> Result<KernelFrame, Error>`
    /// Verified: `pub fn alloc(&mut self) -> Result<KernelFrame, Error>`
    ///
    /// The `clear: bool` parameter is omitted because clearing requires byte-level
    /// memory access via `DerefMut` (which uses `unsafe { from_raw_parts_mut(...) }`).
    /// Verus cannot verify unsafe code. Clearing is orthogonal to allocation safety:
    /// zeroing memory does not affect double-allocation, aliasing, or liveness.
    ///
    /// The allocation logic is identical: both versions allocate a single frame from
    /// the underlying bitmap and wrap it in a `KernelFrame`.
    ///
    /// # Returns
    ///
    /// On success, a KernelFrame containing the allocated frame address is returned.
    /// On failure (pool exhausted), an error is returned.
    pub fn alloc(&mut self) -> (result: Result<KernelFrame, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity() == old(self)@.capacity(),
            // Pool ID is preserved.
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // Converse: If no free frame, allocation fails.
            !old(self)@.has_free_frame() ==> result is Err,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let kframe = result->Ok_0;
                let frame_idx: int = kframe@.frame_number;
                // The frame satisfies its invariant.
                &&& kframe.inv()
                // The frame address is page-aligned (exposed for external modules).
                &&& kframe.spec_is_aligned()
                // The frame index is valid.
                &&& 0 <= frame_idx < self@.capacity()
                // The frame is now allocated.
                &&& self@.is_allocated(frame_idx)
                // The frame was not previously allocated.
                &&& !old(self)@.is_allocated(frame_idx)
                // PROVENANCE: Frame carries this pool's ID.
                &&& kframe@.pool_id == self@.id()
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity() && i != frame_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // EXPLICIT COUNT: exactly one more frame allocated.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + 1,
            // On failure: allocation state unchanged.
            result is Err ==> {
                &&& self@.capacity() == old(self)@.capacity()
                &&& self@.id() == old(self)@.id()
                &&& self@.base() == old(self)@.base()
                &&& forall|i: int| 0 <= i < self@.capacity() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
    {
        match self.frame_allocator.alloc() {
            Ok(addr) => {
                let kframe: KernelFrame = KernelFrame::new_internal(addr, self.pool_id);
                Ok(kframe)
            },
            Err(error) => Err(error),
        }
    }

    //==============================================================================================

    /// Books a contiguous range of frames in the kernel frame pool.
    ///
    /// # Equivalence to Original
    ///
    /// Original `KpoolInner::alloc_range(count)`: Searches for a free contiguous range
    /// via `bitmap.alloc_range(count)` and returns `Vec<FrameAddress>`.
    ///
    /// This verified version splits the original into two operations:
    ///   - `alloc_contiguous(count)`: Searches for and allocates a free range (original search semantics).
    ///   - `alloc_range(start_frame, count)`: Books a specific range (caller specifies start).
    ///
    /// This separation enables clearer verification: the search logic is isolated in
    /// `alloc_contiguous`, while `alloc_range` has simpler preconditions (all frames
    /// in range must be free). The combined behavior of `alloc_contiguous` is
    /// functionally equivalent to the original `alloc_range`.
    ///
    /// # Parameters
    ///
    /// - `start_frame`: Starting frame index (must be free).
    /// - `count`: Number of contiguous frames to allocate.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned.
    pub fn alloc_range(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity(),
            // All frames in range must be initially free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: alloc_range always succeeds when preconditions are met.
            result is Ok,
            // All frames in range are now allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                self@.is_allocated(i),
            // All frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity()) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // EXPLICIT COUNT: allocated count increases by exactly `count`.
            self@.num_allocated() == old(self)@.num_allocated() + count as int,
    {
        // Connect Kpool's view to FrameAllocator's view.
        proof {
            assert forall|i: int| start_frame as int <= i < start_frame as int + count as int
                implies !self.frame_allocator@.is_allocated(i)
            by {
                assert(!self@.is_allocated(i));
            }
        }
        self.frame_allocator.alloc_range_checked(start_frame, count)
    }

    //==============================================================================================

    /// Allocates a contiguous range of frames by searching for a free range.
    ///
    /// # Description
    ///
    /// This is a verification helper that implements the search-and-allocate logic
    /// from the original `KpoolInner::alloc_range()` / `Kpool::alloc_many()`.
    /// The `alloc_many()` wrapper provides the original API name.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of contiguous frames to allocate.
    ///
    /// # Returns
    ///
    /// On success, returns the starting frame index of the allocated range.
    pub fn alloc_contiguous(&mut self, count: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            count > 0,
            count as int <= old(self)@.capacity(),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // On success: a valid contiguous range is allocated.
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.capacity()
                &&& start + count as int <= self@.capacity()
                // All frames in range were previously free.
                &&& forall|i: int| start <= i < start + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count as int ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count as int <= i < self@.capacity()) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Count tracking on success.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + count as int,
            // On failure: allocation state unchanged.
            result is Err ==> {
                &&& self@.capacity() == old(self)@.capacity()
                &&& self@.id() == old(self)@.id()
                &&& self@.base() == old(self)@.base()
                &&& forall|i: int| 0 <= i < self@.capacity() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Liveness for count=1.
            (count == 1 && old(self)@.has_free_frame()) ==> result is Ok,
    {
        match self.frame_allocator.alloc_contiguous_range(count) {
            Ok(start) => {
                proof {
                    // Connect FrameAllocator view to Kpool view.
                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies !old(self)@.is_allocated(i)
                    by {
                        assert(!old(self).frame_allocator@.is_allocated(i));
                    }

                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies self@.is_allocated(i)
                    by {
                        assert(self.frame_allocator@.is_allocated(i));
                    }

                    assert forall|i: int|
                        (0 <= i < start as int || start as int + count as int <= i < self@.capacity())
                        implies self@.is_allocated(i) == old(self)@.is_allocated(i)
                    by {
                        assert(self.frame_allocator@.is_allocated(i) == old(self).frame_allocator@.is_allocated(i));
                    }
                }
                Ok(start)
            },
            Err(e) => Err(e),
        }
    }

    //==============================================================================================

    /// Allocates a contiguous range of frames from the kernel frame pool.
    ///
    /// # Equivalence to Original
    ///
    /// Original: `pub fn alloc_many(&mut self, clear: bool, count: usize) -> Result<Vec<KernelFrame>, Error>`
    ///
    /// This function matches the original `Kpool::alloc_many()` allocation semantics:
    /// it searches for and allocates a contiguous range of `count` frames, delegating
    /// to `alloc_contiguous()` internally.
    ///
    /// ## Differences from original (Verus limitations):
    ///
    /// 1. **`clear` parameter omitted**: Clearing requires unsafe `DerefMut` byte access
    ///    which Verus cannot verify. Clearing is orthogonal to allocation safety.
    /// 2. **Return type is `Result<usize, Error>`** instead of `Result<Vec<KernelFrame>, Error>`:
    ///    Constructing `Vec<KernelFrame>` requires `Rc<RefCell<KpoolInner>>` cloning for
    ///    each frame, which Verus cannot model. The returned `usize` is the starting
    ///    frame index; each frame in `[start, start + count)` is allocated and contiguous.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of contiguous frames to allocate.
    ///
    /// # Returns
    ///
    /// On success, returns the starting frame index of the allocated contiguous range.
    /// On failure (no contiguous range available), returns an error.
    pub fn alloc_many(&mut self, count: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            count > 0,
            count as int <= old(self)@.capacity(),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            self@.base() == old(self)@.base(),
            // On success: a valid contiguous range is allocated.
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.capacity()
                &&& start + count as int <= self@.capacity()
                // All frames in range were previously free.
                &&& forall|i: int| start <= i < start + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count as int ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count as int <= i < self@.capacity()) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Count tracking on success.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + count as int,
            // On failure: allocation state unchanged.
            result is Err ==> {
                &&& self@.capacity() == old(self)@.capacity()
                &&& self@.id() == old(self)@.id()
                &&& self@.base() == old(self)@.base()
                &&& forall|i: int| 0 <= i < self@.capacity() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Liveness for count=1.
            (count == 1 && old(self)@.has_free_frame()) ==> result is Ok,
    {
        // Delegates to alloc_contiguous which implements the original search semantics.
        self.alloc_contiguous(count)
    }

    //==============================================================================================

    /// Allocates multiple frames from the kernel frame pool.
    ///
    /// # Note
    ///
    /// This function allocates `count` individual frames that are **not necessarily
    /// contiguous**. This is a verification-specific API variant. For original
    /// `alloc_many()` semantics (contiguous allocation), use `alloc_many()` or
    /// `alloc_contiguous()` instead.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of frames to allocate (must be > 0).
    ///
    /// # Returns
    ///
    /// Upon success, a ghost sequence of allocated frame indices is returned.
    pub fn alloc_noncontiguous(&mut self, count: usize) -> (result: Result<Ghost<Seq<int>>, Error>)
        requires
            old(self).inv(),
            count > 0,
            // Precondition: must have at least `count` free frames.
            old(self)@.num_allocated() + count as int <= old(self)@.capacity(),
        ensures
            self.inv(),
            // Capacity and region are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.base() == old(self)@.base(),
            self@.id() == old(self)@.id(),
            // Liveness: With precondition satisfied, allocation succeeds.
            result is Ok,
            // On success: exactly count new frames are allocated.
            result is Ok ==> {
                let frame_indices = result->Ok_0@;
                // Correct number of frames returned.
                &&& frame_indices.len() == count as int
                // All frame indices are valid and newly allocated.
                &&& forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < frame_indices.len() ==> {
                        let frame_idx = frame_indices[i];
                        &&& 0 <= frame_idx < self@.capacity()
                        &&& self@.is_allocated(frame_idx)
                        &&& !old(self)@.is_allocated(frame_idx)
                    }
                // All frame indices are distinct (but NOT necessarily contiguous).
                &&& forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                    0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                    frame_indices[i] != frame_indices[j]
            },
            // EXPLICIT COUNT: exactly count more frames allocated.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + count as int,
            // MONOTONICITY: previously allocated frames remain allocated.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && old(self)@.is_allocated(i) ==>
                self@.is_allocated(i),
    {
        let ghost original_self: Kpool = *self;
        let ghost original_capacity: int = self@.capacity();
        let ghost original_num_allocated: int = self.spec_num_allocated();

        let ghost mut frame_indices: Seq<int> = Seq::empty();
        let mut allocated_count: usize = 0;

        while allocated_count < count
            invariant
                self.inv(),
                original_self.inv(),
                self@.capacity() == original_self@.capacity(),
                self@.capacity() == original_capacity,
                // Pool ID is preserved.
                self@.id() == original_self@.id(),
                self@.base() == original_self@.base(),
                // Precondition: enough capacity for all remaining allocations.
                original_num_allocated + count as int <= original_capacity,
                // Loop bounds.
                0 <= allocated_count <= count,
                // Frame indices collected so far.
                frame_indices.len() == allocated_count as int,
                // Allocation count tracking.
                self.spec_num_allocated() == original_num_allocated + allocated_count as int,
                // Remaining room for more allocations.
                self.spec_num_allocated() + (count - allocated_count) as int <= self@.capacity(),
                // MONOTONICITY: frames allocated in original_self remain allocated.
                forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity() && original_self@.is_allocated(i) ==>
                    self@.is_allocated(i),
                // All collected frame indices are valid.
                forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < frame_indices.len() ==> {
                        let frame_idx = frame_indices[i];
                        &&& 0 <= frame_idx < self@.capacity()
                        &&& self@.is_allocated(frame_idx)
                        &&& !original_self@.is_allocated(frame_idx)
                    },
                // All frame indices are distinct.
                forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                    0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                    frame_indices[i] != frame_indices[j],
            decreases
                count - allocated_count,
        {
            let ghost prev_self: Kpool = *self;
            let ghost prev_frame_indices: Seq<int> = frame_indices;
            let ghost prev_len: int = prev_frame_indices.len() as int;

            // We need has_free_frame() for alloc to succeed.
            proof {
                assert(self.spec_num_allocated() < self@.capacity());
                self.frame_allocator.lemma_can_allocate_implies_has_free_frame();
            }

            let kframe: KernelFrame = match self.alloc() {
                Ok(f) => f,
                Err(_) => {
                    proof { assert(false); }
                    return Err(Error::new(ErrorCode::OutOfMemory, "unexpected"));
                },
            };

            let ghost new_frame_idx: int = kframe@.frame_number;

            proof {
                // The new frame is distinct from all previously allocated frames.
                assert forall|k: int| 0 <= k < prev_len
                    implies prev_frame_indices[k] != new_frame_idx
                by {
                    assert(prev_self@.is_allocated(prev_frame_indices[k]));
                    assert(!prev_self@.is_allocated(new_frame_idx));
                }

                // Update frame_indices.
                frame_indices = prev_frame_indices.push(new_frame_idx);

                // For old elements (i < prev_len):
                assert forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < prev_len
                    implies {
                        let idx: int = frame_indices[i];
                        &&& 0 <= idx < self@.capacity()
                        &&& self@.is_allocated(idx)
                        &&& !original_self@.is_allocated(idx)
                    }
                by {
                    let idx: int = frame_indices[i];
                    assert(idx == prev_frame_indices[i]);
                    assert(prev_self@.is_allocated(idx));
                }

                // For the new element (i == prev_len):
                assert({
                    let idx: int = frame_indices[prev_len as int];
                    &&& 0 <= idx < self@.capacity()
                    &&& self@.is_allocated(idx)
                    &&& !original_self@.is_allocated(idx)
                }) by {
                    assert(frame_indices[prev_len as int] == new_frame_idx);
                    assert(!prev_self@.is_allocated(new_frame_idx));
                }
            }
            allocated_count = allocated_count + 1;
        }

        Ok(Ghost(frame_indices))
    }

    //==============================================================================================

    /// Frees a frame that was previously allocated from the kernel frame pool.
    ///
    /// # Equivalence to Original
    ///
    /// Original `KpoolInner::free(addr: FrameAddress)`: Computes frame index from
    /// address and calls `bitmap.clear(index)`. Called implicitly via `Drop` trait.
    ///
    /// This verified version takes `KernelFrame` instead of `FrameAddress` to
    /// enable provenance checking (the frame's `pool_id` must match the pool's `id`).
    /// The original achieves this via `Rc<RefCell<KpoolInner>>` identity, which
    /// Verus cannot model. The core logic (mark frame as free in bitmap) is identical.
    ///
    /// The original uses RAII (`Drop` trait) for automatic deallocation; this version
    /// requires explicit `free()` calls because Verus cannot verify `Drop` with
    /// `Rc<RefCell>`.
    ///
    /// # Parameters
    ///
    /// - `kframe`: Kernel frame to be freed.
    ///
    /// # Returns
    ///
    /// On success, `Ok(())` is returned.
    pub fn free(&mut self, kframe: KernelFrame) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            kframe.inv(),
            kframe@.frame_number < old(self)@.capacity(),
            old(self)@.is_allocated(kframe@.frame_number),
            // PROVENANCE: Frame must belong to this pool.
            kframe@.pool_id == old(self)@.id(),
        ensures
            self.inv(),
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            // On success: the frame is freed.
            !self@.is_allocated(kframe@.frame_number),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && i != kframe@.frame_number ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // Count decremented by 1.
            self@.num_allocated() == old(self)@.num_allocated() - 1,
    {
        self.frame_allocator.free(kframe.address())
    }

    //==============================================================================================

    /// Frees a contiguous range of frames.
    ///
    /// # Description
    ///
    /// Verification helper for batch deallocation. The original uses individual
    /// `Drop` calls on each `KernelFrame` in a `Vec`. This provides equivalent
    /// batch-free semantics without RAII.
    ///
    /// # Parameters
    ///
    /// - `start_frame`: Start frame index (inclusive).
    /// - `count`: Number of frames to free.
    pub fn free_range(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity(),
            // All frames in range must be allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: always succeeds when preconditions met.
            result is Ok,
            // All frames in range are now free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !self@.is_allocated(i),
            // All frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity()) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // COUNT: allocated count decreases by exactly `count`.
            self@.num_allocated() == old(self)@.num_allocated() - count as int,
    {
        // Connect Kpool's view to FrameAllocator's view.
        proof {
            assert forall|i: int| start_frame as int <= i < start_frame as int + count as int
                implies self.frame_allocator@.is_allocated(i)
            by {
                assert(self@.is_allocated(i));
            }
        }
        self.frame_allocator.free_range(start_frame, count)
    }

    //==============================================================================================

    /// Frees a contiguous range of frames with ghost index validation.
    ///
    /// # Description
    ///
    /// Verification helper that extends `free_range` with ghost-level index
    /// validation. Useful for bridging `alloc_noncontiguous` ghost results
    /// back to contiguous free operations.
    ///
    /// # Parameters
    ///
    /// - `start_frame`: Starting frame index.
    /// - `frame_indices`: Ghost sequence of frame indices.
    /// - `count`: Number of frames to free.
    pub fn free_contiguous(&mut self, start_frame: usize, frame_indices: Ghost<Seq<int>>, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            frame_indices@.len() == count as int,
            start_frame as int + count as int <= old(self)@.capacity(),
            // Frame indices must be the contiguous range [start_frame, start_frame + count).
            forall|i: int| #![trigger frame_indices@[i]]
                0 <= i < frame_indices@.len() ==>
                frame_indices@[i] == start_frame as int + i,
            // All frames in range must be allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: always succeeds when preconditions met.
            result is Ok,
            // All frames in the sequence are now free.
            forall|i: int| #![trigger frame_indices@[i]]
                0 <= i < frame_indices@.len() ==>
                !self@.is_allocated(frame_indices@[i]),
            // All frames in range are now free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !self@.is_allocated(i),
            // Frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity()) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // COUNT: allocated count decreases by exactly `count`.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() - count as int,
    {
        // Delegate to free_range since the frame indices are contiguous.
        self.free_range(start_frame, count)
    }
}

} // verus!
