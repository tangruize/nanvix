# Nanvix Verus Verification

This directory contains Verus-verified versions of core Nanvix kernel memory
management components, unified into a single crate for formal verification.

## Overview

[Verus](https://github.com/verus-lang/verus) is a tool for verifying the
correctness of Rust code using formal methods. This crate provides verified
implementations of Nanvix's core allocators and memory management primitives,
proving critical memory safety properties at compile time.

## Directory Structure

```
verus/
├── lib.rs              # Crate root - module declarations
├── error.rs            # Error types (ErrorCode, Error)
├── raw_array.rs        # Raw memory array with verified specifications
├── bitmap.rs           # Bitmap allocator (bit-level allocation tracking)
├── slab.rs             # Slab allocator (fixed-size block allocation)
├── kheap.rs            # Kernel heap allocator (multi-slab management)
├── frame_address.rs    # Frame address and frame number types
├── frame.rs            # Frame allocator core (page-granularity allocation)
├── upool.rs            # User frame pool (user-space memory management)
├── kpool.rs            # Kernel frame pool (kernel-space memory management)
└── README.md           # This file
```

## Module Hierarchy

Each module builds on lower-level verified modules, forming a layered architecture:

```
error                           # Base error types
  └── raw_array                 # Safe raw memory abstraction
       └── bitmap               # Bit-level allocation tracking
            ├── slab            # Fixed-size block allocator
            │    └── kheap      # Multi-slab kernel heap
            └── frame_address   # Page-aligned address types
                 └── frame      # Page-granularity frame allocator
                      ├── upool # User-space frame pool
                      └── kpool # Kernel-space frame pool
```

## Running Verification

### Prerequisites

Install Verus following the [official instructions](https://github.com/verus-lang/verus).

### Full Verification

```bash
cd verus && verus --crate-type lib lib.rs
# verification results: 327 verified, 0 errors
```

### Per-Module Verification

For faster iteration during development:

```bash
cd verus && verus --crate-type lib lib.rs --verify-module kpool
cd verus && verus --crate-type lib lib.rs --verify-module upool
cd verus && verus --crate-type lib lib.rs --verify-module frame
cd verus && verus --crate-type lib lib.rs --verify-module bitmap
cd verus && verus --crate-type lib lib.rs --verify-module slab
cd verus && verus --crate-type lib lib.rs --verify-module kheap
cd verus && verus --crate-type lib lib.rs --verify-module raw_array
cd verus && verus --crate-type lib lib.rs --verify-module frame_address
cd verus && verus --crate-type lib lib.rs --verify-module error
```

## Module Summaries

### error.rs

Defines error types used throughout the verification crate.

- **`ErrorCode`**: Enum of error codes (InvalidArgument, OutOfMemory, ResourceBusy, BadAddress).
- **`Error`**: Combines an error code with a reason string.

**Original Source**: `src/libs/sys/src/error.rs`

### raw_array.rs

Provides a verified specification for raw memory arrays with external-body
implementations for unsafe operations.

**Key Abstractions**:
- **`RawArray<T>`**: Fixed-size array backed by raw memory.
- **`RawArrayView<T>`**: Abstract specification as `Seq<T>`.

**Memory Safety Model**:
1. **Length invariance**: Array length never changes after construction.
2. **Element isolation**: Writing to one index doesn't affect others.
3. **Bounds safety**: All accesses are within bounds.
4. **Initial state**: New arrays are zero-initialized.

**Verified Lemmas**:
- `lemma_update_preserves_len`: Update preserves array length.
- `lemma_update_only_changes_index`: Update only modifies target index.
- `lemma_update_sets_index`: Update correctly sets the target value.
- `lemma_set_commutes`: Updates to different indices commute.
- `lemma_set_overwrite`: Consecutive updates to same index keep last value.

**Original Source**: `src/libs/raw-array/src/lib.rs`

### bitmap.rs

Implements a verified bitmap allocator for tracking allocation status of
fixed-size resources.

**Key Abstractions**:
- **`Bitmap`**: Tracks used/free status of bits.
- **`BitmapView`**: Abstract specification as `Seq<bool>`.

**Verified Properties**:
- Allocation returns an unset (free) bit.
- Clear requires the bit to be set (prevents double-free).
- Usage count accurately tracks set bits.
- **Liveness**: Allocation succeeds when free bits exist.

**API**:
- `Bitmap::new(number_of_bits)` - Create new bitmap.
- `Bitmap::alloc()` - Allocate a free bit, returns index.
- `Bitmap::clear(index)` - Free a previously allocated bit.
- `Bitmap::number_of_bits()` - Get total capacity.
- `Bitmap::usage()` - Get count of allocated bits.

**Original Source**: `src/libs/bitmap/src/lib.rs`

### slab.rs

Implements a verified slab allocator for fixed-size block allocation.

**Key Abstractions**:
- **`Slab`**: Manages fixed-size memory blocks.
- **`SlabView`**: Abstract specification tracking allocated blocks.

**Memory Layout**:
```
+-------------------+--------------------------------------+
| Index Blocks      | Data Blocks                          |
+-------------------+--------------------------------------+
```

**Verified Properties**:
- Block allocation/deallocation correctness.
- Invariant preservation across all operations.
- No overlapping allocations between blocks.
- Buffer bounds checking for all operations.

**API**:
- `Slab::new(ptr, len, block_size)` - Create slab from memory region.
- `Slab::alloc()` - Allocate a block, returns address.
- `Slab::free(addr)` - Free a previously allocated block.
- `Slab::capacity()` - Get total number of blocks.
- `Slab::used()` - Get count of allocated blocks.

**Original Source**: `src/libs/slab/src/lib.rs`

### kheap.rs

Implements a verified kernel heap allocator that manages multiple slabs of
different block sizes.

**Key Abstractions**:
- **`Kheap`**: Multi-slab kernel heap.
- **`SlabSize`**: Enum of supported block sizes (8, 16, 32, 64, 128, 256, 512, 4096 bytes).

**Memory Layout**:
```
+----------+----------+----------+----------+----------+----------+----------+----------+
| Slab 8   | Slab 16  | Slab 32  | Slab 64  | Slab 128 | Slab 256 | Slab 512 | Slab4096 |
+----------+----------+----------+----------+----------+----------+----------+----------+
```

**Verified Properties**:
- Correct slab selection based on allocation size.
- Multi-slab disjointness (no overlap between slabs).
- Allocation validity within correct slab's range.
- Deallocation targets the correct slab.
- Invariant preservation across operations.

**API**:
- `Kheap::new(ptr, len)` - Create heap from memory region.
- `Kheap::alloc(size)` - Allocate memory of given size.
- `Kheap::free(addr, size)` - Free previously allocated memory.

**Original Source**: `src/kernel/src/mm/kheap.rs`

### frame_address.rs

Provides verified types for representing page-aligned addresses and frame numbers.

**Key Abstractions**:
- **`FrameNumber`**: Represents a frame index (0 to MAX_FRAME_NUMBER).
- **`FrameAddress`**: Represents a page-aligned physical address.
- **`PageAlignedPhysAddr`**: Simplified page-aligned address type.

**Constants**:
- `FRAME_SIZE = 4096` (4 KB pages).
- `MAX_FRAME_NUMBER = 0xFFFF_FFFF / FRAME_SIZE` (32-bit address space).

**Verified Properties**:
- Frame addresses are always page-aligned.
- Conversion between frame number and address is correct.
- No overflow in address calculations.

**Original Source**: `src/kernel/src/hal/mem/mod.rs` (FrameAddress, PageAligned types)

### frame.rs

Implements the core verified frame allocator for page-granularity memory
allocation.

**Key Abstractions**:
- **`FrameAllocator`**: Manages physical memory frames.
- **`FrameAllocatorView`**: Abstract specification with `Set<int>` of allocated frames.

**Verified Properties**:
- **Frame disjointness**: Different frames have non-overlapping memory regions.
- **No memory aliasing**: All allocated frames are mutually disjoint.
- **Liveness guarantees**: Operations succeed when preconditions are met.
- **Count tracking**: Accurate tracking of allocated/free frames.
- **Contiguous search**: Ability to find contiguous free ranges.

**API**:
- `FrameAllocator::new(bitmap)` - Create from bitmap.
- `FrameAllocator::alloc()` - Allocate single frame.
- `FrameAllocator::free(index)` - Free a frame by index.
- `FrameAllocator::alloc_contiguous(count)` - Search for contiguous range.
- `FrameAllocator::capacity()` - Get total frame count.

**Original Source**: `src/kernel/src/mm/phys/frame.rs`

### upool.rs

Implements a verified user frame pool for managing user-space memory frames.

**Key Abstractions**:
- **`Upool`**: Wraps FrameAllocator for user-space allocation.
- **`UpoolView`**: Abstract specification.
- **`UserFrame`**: Wrapper around FrameAddress for allocated frames.
- **`FramePermission`**: Read-only or read-write access tracking.

**Verified Properties**:
- **No Double Allocation**: Frames can only be allocated if currently free.
- **No Double Free**: Frames can only be freed if currently allocated.
- **No Memory Aliasing**: All allocated frames have disjoint memory regions.
- **Valid Frame Indices**: All frames have indices within [0, capacity).
- **Liveness**: Allocation succeeds when free frames exist.

**API**:
- `Upool::new(frame_allocator)` - Create pool from allocator.
- `Upool::alloc()` - Allocate a single frame.
- `Upool::alloc_many(count)` - Allocate multiple frames (ghost indices).
- `Upool::free(uframe)` - Free a previously allocated frame.
- `Upool::capacity()` - Get total number of frames.

**alloc_many Loop Invariants**:
1. **Monotonicity**: Frames allocated in original state remain allocated.
2. **Distinctness**: All collected frame indices are unique.
3. **Freshness**: All frames were unallocated in original state.
4. **Count**: Exactly `count` frames are allocated.

**Original Source**: `src/kernel/src/mm/phys/upool.rs`

### kpool.rs

Implements a verified kernel frame pool for managing kernel-space memory frames
with provenance tracking.

**Key Abstractions**:
- **`Kpool`**: Wraps FrameAllocator for kernel-space allocation.
- **`KpoolView`**: Abstract specification with pool_id and base_addr.
- **`KernelFrame`**: Wrapper with pool_id for provenance tracking.

**Verified Properties**:
- **No Double Allocation**: Frames can only be allocated if currently free.
- **No Double Free**: Frames can only be freed if currently allocated.
- **No Memory Aliasing**: All allocated frames have disjoint memory regions.
- **Valid Frame Indices**: All frames have indices within [0, capacity).
- **Provenance Tracking**: Frames can only be freed to their originating pool.
- **Contiguous Range Validity**: Range allocations return contiguous indices.
- **Count Correctness**: All operations maintain exact frame counts.
- **Liveness**: Operations succeed when preconditions are met.

**API**:
- `Kpool::new(frame_allocator, pool_id)` - Create pool with unique identifier.
- `Kpool::alloc()` - Allocate single frame (with pool_id).
- `Kpool::alloc_contiguous(count)` - Search for and allocate contiguous range.
- `Kpool::alloc_range(start, count)` - Book a specific contiguous range.
- `Kpool::alloc_noncontiguous(count)` - Allocate multiple (non-contiguous) frames.
- `Kpool::free(kframe)` - Free a frame (requires matching pool_id).
- `Kpool::free_range(start, count)` - Free a contiguous range.
- `Kpool::free_contiguous(start, indices, count)` - Free with ghost validation.
- `Kpool::capacity()` - Get total frame count.
- `Kpool::get_pool_id()` - Get the pool identifier.

**Provenance Tracking**:
The `pool_id` is stored in both `Kpool` and `KernelFrame`. When calling `free()`,
the precondition `kframe.spec_pool_id() == self@.id()` ensures frames are only
freed to their originating pool, preventing cross-pool aliasing bugs.

**Abstraction Differences from Original**:
| Aspect | Original | Verified |
|--------|----------|----------|
| Contiguous allocation | `alloc_many(count)` | `alloc_contiguous(count)` |
| Range booking | N/A | `alloc_range(start, count)` |
| Non-contiguous batch | N/A | `alloc_noncontiguous(count)` |
| Memory clearing | `clear` parameter | Orthogonal to safety |
| Deallocation | RAII via Drop | Explicit `free()` calls |
| Ownership | `Rc<RefCell<>>` | Single-owner semantics |

**Original Source**: `src/kernel/src/mm/phys/kpool.rs`

## Verification Summary

| Module           | Lines | Purpose                          | Key Properties |
|------------------|-------|----------------------------------|----------------|
| `error`          | ~80   | Error types                      | Type safety |
| `raw_array`      | ~735  | Raw memory array                 | Memory safety, bounds checking, zero-init |
| `bitmap`         | ~2000 | Bitmap allocator                 | Allocation correctness, liveness |
| `slab`           | ~700  | Slab allocator                   | Block allocation, no overlap, bounds |
| `kheap`          | ~500  | Kernel heap allocator            | Multi-slab disjointness, size mapping |
| `frame_address`  | ~180  | Frame address types              | Alignment, conversion correctness |
| `frame`          | ~1300 | Frame allocator                  | No aliasing, liveness, contiguous search |
| `upool`          | ~400  | User frame pool                  | Memory safety, alloc/free/alloc_many |
| `kpool`          | ~600  | Kernel frame pool                | Memory safety, provenance, contiguous |

**Total: 327 verified, 0 errors**

## Verification Approach

### Trusted Code Boundary

The verification approach minimizes the trusted computing base (TCB):

1. **Spec functions**: Fully verified mathematical specifications.
2. **Lemmas and proofs**: Fully verified logical reasoning.
3. **Exec functions with specifications**: Verified against specs.
4. **External body functions**: Trusted implementations for:
   - Raw memory allocation (`alloc`, `dealloc`).
   - Pointer dereferencing (unsafe Rust operations).
   - View function implementations that touch raw memory.

### Abstraction Patterns

- **View types** (e.g., `BitmapView`, `FrameAllocatorView`): Abstract specifications.
- **Ghost code**: Proof-only code that doesn't affect execution.
- **Invariant predicates**: Properties maintained by all operations.
- **Postconditions**: Guarantees provided by each operation.

## Design Notes

### Why Single-Owner Semantics?

The original Nanvix implementation uses `Rc<RefCell<>>` for shared ownership of
pools. For verification, we model pools with exclusive ownership because:

1. **Clearer specifications**: Single-owner semantics avoid aliasing complexity.
2. **Stronger guarantees**: Exclusive access simplifies invariant reasoning.
3. **Core correctness**: The Rc/RefCell pattern is a runtime mechanism that
   doesn't affect the core allocation invariants being verified.

### Why Explicit free() Instead of Drop?

Using explicit `free()` calls instead of RAII/Drop semantics because:

1. **Proof clarity**: Explicit calls make proof obligations visible.
2. **Precondition checking**: Callers must prove frames are allocated.
3. **Specification simplicity**: No need to model automatic deallocation.

## Development Workflow

1. **Modify implementation** in the target `.rs` file.
2. **Run verification**: `verus --crate-type lib lib.rs`.
3. **Fix errors** until verification passes.
4. **Run module-specific verification** for faster iteration.
5. **Update this README** if adding new modules or changing APIs.

## Original Source File Mapping

| Verified Module  | Original Nanvix Source |
|------------------|------------------------|
| `error.rs`       | `src/libs/sys/src/error.rs` |
| `raw_array.rs`   | `src/libs/raw-array/src/lib.rs` |
| `bitmap.rs`      | `src/libs/bitmap/src/lib.rs` |
| `slab.rs`        | `src/libs/slab/src/lib.rs` |
| `kheap.rs`       | `src/kernel/src/mm/kheap.rs` |
| `frame_address.rs` | `src/kernel/src/hal/mem/mod.rs` |
| `frame.rs`       | `src/kernel/src/mm/phys/frame.rs` |
| `upool.rs`       | `src/kernel/src/mm/phys/upool.rs` |
| `kpool.rs`       | `src/kernel/src/mm/phys/kpool.rs` |

## References

- [Verus Documentation](https://verus-lang.github.io/verus/guide/)
- [Verus GitHub Repository](https://github.com/verus-lang/verus)
- [Nanvix Project](https://github.com/nanvix/nanvix)
