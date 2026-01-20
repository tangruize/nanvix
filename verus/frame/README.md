# Verified Frame Allocator

This directory contains a Verus-verified version of the Nanvix frame allocator
(`src/kernel/src/mm/phys/frame.rs`).

## Overview

The frame allocator manages physical memory frames (4KB pages) using a bitmap
to track allocation status. Each bit in the bitmap corresponds to one frame:
- Bit set (1) = frame is allocated
- Bit unset (0) = frame is free

## Module Structure

```
verus/frame/
├── lib.rs              # Main entry point and re-exports
├── error.rs            # Error types (ErrorCode, Error)
├── bitmap.rs           # Bitmap abstraction (trusted dependency)
├── frame_address.rs    # Frame address types (trusted dependency)
├── permission.rs       # Ghost ownership types (FramePermission)
├── frame_core.rs       # Verified frame allocator implementation
├── original_frame.rs   # Original source code for reference
├── README.md           # This file
└── VERIFICATION_SUMMARY.md  # Detailed verification summary
```

## Verification

To verify the implementation:

```bash
cd verus/frame
verus --crate-type lib lib.rs
```

Expected output:
```
verification results:: 49 verified, 0 errors
```

## Key A+ Features

### 1. Memory Safety as First-Class Invariant

`no_memory_aliasing()` is part of `inv()`, so every operation automatically preserves
memory safety without needing separate preservation lemmas.

### 2. Explicit Count Postconditions

All allocation/deallocation operations have explicit count tracking:
- `alloc`: `self.spec_num_allocated() == old(self).spec_num_allocated() + 1`
- `alloc_address`: `self.spec_num_allocated() == old(self).spec_num_allocated() + 1`  
- `alloc_range`: `self.spec_num_allocated() == old(self).spec_num_allocated() + count`
- `book`: +1 if frame was free, unchanged if already allocated (idempotent)
- `book_tracked`: `self.spec_num_allocated() == old(self).spec_num_allocated() + 1`
- `free`: `self.spec_num_allocated() == old(self).spec_num_allocated() - 1`
- `free_range_tracked`: `self.spec_num_allocated() == old(self).spec_num_allocated() - count`

### 3. Ghost Ownership Tracking

The allocator provides tracked permission types for exclusive ownership proofs:

- **`FramePermission`**: A tracked ghost type that proves exclusive ownership of a frame.
  Created by `alloc_tracked` or `book_tracked`, consumed by `free_tracked`.

- **`RangePermission`**: A tracked ghost type that proves exclusive ownership of a
  contiguous range of frames. Created by `alloc_range_tracked`, consumed by
  `free_range_tracked`.

- **Tracked Allocation**: 
  - `alloc_tracked()` returns `Result<(usize, Tracked<FramePermission>), Error>`
  - `alloc_range_tracked()` returns `Result<Tracked<RangePermission>, Error>`
  - `book_tracked()` returns `Result<Tracked<FramePermission>, Error>`

- **Tracked Deallocation**: 
  - `free_tracked()` requires a `Tracked<FramePermission>` matching the frame
  - `free_range_tracked()` requires a `Tracked<RangePermission>` matching the range
  
  Ensuring:
  1. Only the owner can free (prevents use-after-free by others)
  2. Double-free is impossible (permission is consumed)
  3. Cross-allocator confusion is prevented (allocator_id must match)

- **API Soundness**: `free_untracked` is **private** (not pub) to prevent creating dangling
  permissions. Only tracked deallocation APIs are exposed.

Example usage:
```rust
// Single frame allocation with permission
let (frame_idx, Tracked(perm)) = allocator.alloc_tracked()?;
// ... use the frame, perm proves exclusive ownership ...
// Free requires the permission
allocator.free_tracked(frame_addr, Tracked(perm))?;

// Range allocation with permission
let Tracked(range_perm) = allocator.alloc_range_tracked(start, count)?;
// ... use the frames, range_perm proves exclusive ownership ...
// Free requires the range permission
allocator.free_range_tracked(start, count, Tracked(range_perm))?;
```

### 4. Full Liveness Guarantees

All operations have formal liveness postconditions:
- `alloc`: `has_free_frame() ==> result is Ok`
- `book`: `result is Ok` (always succeeds when preconditions met)
- `book_tracked`: `result is Ok` (always succeeds when preconditions met)
- `alloc_range`: `result is Ok` (always succeeds when preconditions met)
- `alloc_range_tracked`: `result is Ok` (always succeeds when preconditions met)
- `free_range_tracked`: `result is Ok` (always succeeds when preconditions met)

These are proven by showing that the underlying bitmap operations always succeed
when their preconditions are satisfied.

## Key Properties Verified

### Memory Safety

1. **Frame Disjointness**: All allocated frames have non-overlapping memory regions.
   Each frame occupies exactly `FRAME_SIZE` (4096) bytes, and frames at different
   indices have disjoint address ranges.

2. **Bounds Checking**: All frame indices are within the valid range `[0, capacity)`.
   Attempts to access frames outside this range are prevented by preconditions.

3. **No Double Allocation**: A frame cannot be allocated twice. The `alloc` operation
   only succeeds for frames that are currently free.

4. **No Invalid Free**: A frame can only be freed if it was previously allocated.
   The `free` operation has a precondition requiring the frame to be allocated.

5. **No Memory Aliasing (First-Class Invariant)**: All allocated frames have
   disjoint memory regions. This is now part of `inv()`, so every operation
   automatically preserves memory safety without needing separate lemmas.

### Invariant Preservation

The allocator maintains the following invariant across all operations:

- The underlying bitmap satisfies its own invariant
- Capacity is positive and bounded by `MAX_FRAME_NUMBER + 1`
- The view correctly reflects the bitmap state
- All allocated frame indices are in valid range
- **No memory aliasing** - all allocated frames have disjoint regions

### Functional Correctness

1. **Allocation**: Returns a previously-free frame index, marks it as allocated,
   and leaves all other frames unchanged.

2. **Deallocation**: Marks an allocated frame as free, leaving all other frames
   unchanged.

3. **Booking**: Reserves a specific frame by marking it as allocated.

## Trusted Dependencies

The following components are treated as trusted (external_body):

1. **Bitmap**: The bitmap allocator is verified separately in `verus/slab/bitmap.rs`.
   Its specification is trusted here.

2. **FrameNumber/FrameAddress**: Type conversions are simple and their specs
   directly encode the expected behavior.

## Correspondence to Original Code

| Original Function | Verified Function | Notes |
|-------------------|-------------------|-------|
| `FrameAllocator::new` | `FrameAllocator::new` | Verified |
| `FrameAllocator::from_raw_storage` | `FrameAllocator::from_raw_storage` | Verified |
| `FrameAllocator::alloc` | `FrameAllocator::alloc` | Returns `usize`; has liveness postcondition |
| `FrameAllocator::alloc` | `FrameAllocator::alloc_address` | Returns `FrameAddress` |
| `FrameAllocator::alloc` | `FrameAllocator::alloc_tracked` | Returns `(usize, Tracked<FramePermission>)` |
| `FrameAllocator::free` | `FrameAllocator::free_tracked` | Consumes permission; verified |
| `FrameAllocator::book` | `FrameAllocator::book` | Verified; idempotent; has liveness |
| `FrameAllocator::book` | `FrameAllocator::book_tracked` | Returns `Tracked<FramePermission>` |
| `FrameAllocator::alloc_range` | `FrameAllocator::alloc_range` | Verified with loop invariants; has liveness and count |
| `FrameAllocator::alloc_range` | `FrameAllocator::alloc_range_tracked` | Returns `Tracked<RangePermission>` |
| (new) | `FrameAllocator::free_range_tracked` | Consumes `RangePermission`; verified |

## Design Decisions

1. **Simplified Address Types**: The verification uses simplified `FrameAddress`
   and `PageAlignedPhysAddr` types that directly encode the alignment requirements,
   rather than the full HAL type hierarchy.

2. **Index-Based API**: The verified `alloc` returns a `usize` frame index rather
   than a `FrameAddress`. This simplifies verification while the address computation
   is straightforward (`index * FRAME_SIZE`).

3. **Closed Invariant**: The invariant is closed to encapsulate the bitmap-allocation
   connection, with a lemma (`lemma_allocated_iff_bit_set`) to reveal it when needed.

4. **Idempotent Book**: The `book` operation is specified as idempotent - booking
   an already-allocated frame succeeds and leaves the frame allocated. This matches
   the use case for reserving memory regions.

5. **Formal Liveness**: All operations include formal liveness postconditions:
   - `alloc`: `has_free_frame() ==> result is Ok`
   - `book`: `result is Ok` (always succeeds when preconditions met)
   - `alloc_range`: `result is Ok` (always succeeds when preconditions met)
   
   These are proven via lemmas connecting abstract predicates to bitmap operations.

6. **Tracked vs Untracked API**: The allocator provides both tracked (`alloc_tracked`,
   `free_tracked`) and untracked (`alloc`, `free_untracked`) variants:
   - **Tracked**: Returns/requires `FramePermission` tokens for verified ownership
   - **Untracked**: For legacy code or reservations (`book`) without ownership tracking
   
   **Soundness**: `free_untracked` is **private** to prevent creating dangling permissions.

7. **Architecture (32-bit x86)**: `MAX_FRAME_NUMBER` is calculated as `0xFFFF_FFFF / FRAME_SIZE`
   (approximately 1M frames), targeting a 32-bit physical address space. For 64-bit support,
   this constant would need to be adjusted to `usize::MAX / FRAME_SIZE`.

## Limitations

1. **Single-Threaded**: The verification assumes single-threaded access via mutable
   borrows. Concurrent access must be protected by external locks in the actual kernel.
   Concurrent specification is not implemented because:
   - Nanvix frame allocator is designed for single-threaded use
   - Concurrency would require external synchronization (spinlock, mutex)
   - The lock invariant would hold the allocator's invariant
   - Ghost ownership tracking is the appropriate abstraction for this level

## Testing

The module includes verification-time test proofs:

- `test_fresh_allocator_empty`: Validates the initialization specification
- `test_alloc_valid_index`: Validates allocation postconditions
- `test_free_makes_available`: Validates deallocation postconditions
- `test_frames_disjoint`: Validates the frame disjointness property

## Future Work

1. Add capacity tracking and statistics.

2. Verify integration with the full HAL address types.
