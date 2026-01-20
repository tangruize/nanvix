# Verified Kernel Heap Allocator

This directory contains a Verus-verified version of the Nanvix kernel heap allocator (kheap).

**Last verified: 2026-01-19**

> 📖 **New to this codebase?** Start with [PROOF_GUIDE.md](PROOF_GUIDE.md) for a comprehensive
> introduction to kheap, its correctness specification, and how to read the proof code.

## Files

### Multi-file Structure
- `lib.rs` - Main entry point
- `error.rs` - Error types (trusted)
- `slab.rs` - Slab allocator abstraction (trusted, verified separately in ../slab/)
- `kheap_core.rs` - **Verified kernel heap allocator** (core logic verified)
- `PROOF_GUIDE.md` - **Detailed guide** to understanding the verification

## Verification

```bash
cd verus/kheap
verus --crate-type lib lib.rs
```

**Result: 47 verified, 0 errors**

## Architecture

The kernel heap manages 8 slabs of different block sizes:

```
                         Kernel Heap Memory Layout
+----------+----------+----------+----------+----------+----------+----------+----------+
| Slab 8   | Slab 16  | Slab 32  | Slab 64  | Slab 128 | Slab 256 | Slab 512 | Slab4096 |
+----------+----------+----------+----------+----------+----------+----------+----------+
  8-byte     16-byte    32-byte    64-byte   128-byte   256-byte   512-byte   4096-byte
  blocks     blocks     blocks     blocks    blocks     blocks     blocks     blocks
```

### Size-to-Slab Mapping

| Allocation Size | Slab Used |
|-----------------|-----------|
| 1-8 bytes       | Slab8     |
| 9-16 bytes      | Slab16    |
| 17-32 bytes     | Slab32    |
| 33-64 bytes     | Slab64    |
| 65-128 bytes    | Slab128   |
| 129-256 bytes   | Slab256   |
| 257-512 bytes   | Slab512   |
| 4096 bytes      | Slab4096  |

## Key Properties Verified

### 1. Correct Slab Selection
The `layout_to_slab_size` function correctly maps allocation sizes to the appropriate slab:
- Sizes 1-8 → Slab8 (8-byte blocks)
- Sizes 9-16 → Slab16 (16-byte blocks)
- And so on...

This ensures allocations always get a block of sufficient size.

### 2. Slab Memory Disjointness (`all_slabs_disjoint`)
All 8 slabs manage disjoint memory regions. This means:
- An address valid in Slab8 cannot be valid in Slab16
- No memory overlap between any pair of slabs
- Prevents accidental cross-slab corruption

### 3. Allocation Validity
When `allocate(size)` succeeds:
- The returned address is within the correct slab's region
- The block is marked as allocated in that slab
- The address is aligned to the block size
- Other slabs are unchanged (frame condition)

### 4. Heap Extent Containment (`all_slabs_within_extent`)
All slab data regions are contained within the original heap buffer:
- Every slab's data region is within [base_addr, base_addr + total_size)
- This is established at construction and preserved by operations

### 5. Alignment Guarantees (`all_slabs_aligned`)
All slab data regions are properly aligned:
- Each slab's data_addr is aligned to its block_size
- All returned pointers are aligned to their block size
- E.g., 64-byte blocks are always 64-byte aligned

### 6. Deallocation Correctness
When `deallocate(ptr, size)` succeeds:
- The block is deallocated from the correct slab (based on size)
- Other slabs are unchanged (frame condition)

### 5. Invariant Preservation
All operations preserve the `Kheap::inv()` invariant:
- All 8 slabs maintain their individual invariants
- Each slab has the correct block size (8, 16, 32, 64, 128, 256, 512, 4096)
- All slabs remain disjoint

## Verified Functions

| Function | Properties Verified |
|----------|---------------------|
| `layout_to_slab_size` | Correctly maps size to slab category, returns error for unsupported sizes |
| `SlabSize::as_usize` | Returns correct numeric value for each slab size |
| `Kheap::allocate` | Selects correct slab, returns valid address, preserves invariant, frame condition |
| `Kheap::deallocate` | Targets correct slab, frees block, preserves invariant, frame condition |

## Proven Lemmas

| Lemma | Property |
|-------|----------|
| `lemma_layout_to_slab_correct` | Size ≤ slab_size for all valid mappings |
| `lemma_slab_sizes_partition_space` | Slab sizes are strictly increasing |
| `lemma_allocation_in_correct_slab` | Allocated address is NOT valid in other slabs |
| `lemma_allocation_meets_size_requirement` | Allocated block size ≥ requested size |
| `lemma_deallocation_correct_slab` | Deallocation targets the correct slab |
| `lemma_inv_implies_slab_invs` | Heap invariant implies all slab invariants |
| `lemma_capacity_conserved` | Total capacity is unchanged across operations |

## Invariant (`Kheap::inv`)

The heap invariant ensures these conditions:

1. `slab_8_bytes.inv()` - 8-byte slab is valid (includes alignment)
2. `slab_16_bytes.inv()` - 16-byte slab is valid (includes alignment)
3. `slab_32_bytes.inv()` - 32-byte slab is valid (includes alignment)
4. `slab_64_bytes.inv()` - 64-byte slab is valid (includes alignment)
5. `slab_128_bytes.inv()` - 128-byte slab is valid (includes alignment)
6. `slab_256_bytes.inv()` - 256-byte slab is valid (includes alignment)
7. `slab_512_bytes.inv()` - 512-byte slab is valid (includes alignment)
8. `slab_4096_bytes.inv()` - 4096-byte slab is valid (includes alignment)
9. Block sizes are correct (8, 16, 32, 64, 128, 256, 512, 4096)
10. `all_slabs_disjoint()` - All slab memory regions are disjoint
11. `all_slabs_within_extent()` - All slabs are within [base_addr, base_addr + total_size)
12. `all_slabs_aligned()` - All slab data regions are aligned to their block size
13. `base_addr > 0` and `total_size > 0` - Heap extent is valid

## Verified Tests

### Core Functionality Tests

| Test | Property Verified |
|------|-------------------|
| `test_layout_to_slab_size_verified` | All size ranges map correctly |
| `test_slab_size_as_usize_verified` | SlabSize enum values are correct |
| `test_slab_size_ordering_verified` | Slab sizes form valid ordering |
| `test_spec_layout_to_slab_coverage_verified` | All valid sizes are covered |
| `test_slabs_disjoint_property_verified` | Disjointness property holds |
| `test_address_exclusivity_verified` | Addresses are exclusive to one slab |
| `test_total_capacity_verified` | Total capacity is sum of parts |
| `test_empty_heap_verified` | Empty heap has zero allocations |

### Behavioral Tests (Allocation/Deallocation)

| Test | Property Verified |
|------|-------------------|
| `test_allocation_postconditions_verified` | Allocation returns valid, aligned address of sufficient size |
| `test_allocation_frame_condition_verified` | Allocation in one slab doesn't affect other slabs |
| `test_deallocation_frame_condition_verified` | Deallocation in one slab doesn't affect other slabs |
| `test_invariant_preservation_allocation_verified` | Heap invariant preserved through operations |
| `test_double_allocation_different_addresses_verified` | Two allocations return different addresses |
| `test_allocation_deallocation_roundtrip_verified` | Alloc+dealloc returns allocation count to original |
| `test_size_requirements_all_slabs_verified` | All slab sizes meet or exceed requested sizes |
| `test_alignment_requirements_all_slabs_verified` | All slabs are aligned to their block size |
| `test_heap_extent_respected_verified` | All valid addresses are within heap extent |

## Trusted Dependencies

The kheap verification relies on:

1. **Slab allocator** (`slab.rs`) - Verified separately in `../slab/slab_core.rs`
2. **Error types** (`error.rs`) - Simple data types, no complex logic

The Slab abstraction is marked with `external_body` because:
- It is verified independently in the slab module
- The kheap verification focuses on multi-slab management
- This separation of concerns simplifies the proof structure

## Memory Safety Properties

### 1. No Cross-Slab Aliasing
Due to `all_slabs_disjoint()`, addresses allocated from different slabs never overlap:
```
Slab8.region ∩ Slab16.region = ∅
Slab8.region ∩ Slab32.region = ∅
... (all 28 pairs are disjoint)
```

### 2. Correct Size Guarantees
The allocated block is always ≥ the requested size:
```
allocate(17) → Slab32 (32-byte block) → 32 ≥ 17 ✓
allocate(100) → Slab128 (128-byte block) → 128 ≥ 100 ✓
```

### 3. Frame Condition
Operations on one slab don't affect other slabs:
```
allocate(8) modifies Slab8 only
slab_16, slab_32, ..., slab_4096 are unchanged
```

## Comparison with Original Code

| Aspect | Original `kheap.rs` | Verified `kheap_core.rs` |
|--------|---------------------|--------------------------|
| Lines of code | 238 | ~1100 |
| Slab selection | Pattern matching | Pattern matching + proof |
| Disjointness | Implicit | Explicitly verified |
| Frame condition | Not specified | Proven for all operations |
| Error handling | Runtime errors | Preconditions + runtime errors |

## Usage

```bash
# Verify the kheap module
cd verus/kheap
verus --crate-type lib lib.rs

# Expected output:
# verification results:: 47 verified, 0 errors
```

## Differences from Original

The verified version has these key differences:

1. **Slab abstraction**: Uses a simplified trusted Slab interface
2. **No GlobalAlloc**: Does not implement the `GlobalAlloc` trait (unsafe FFI boundary)
3. **Explicit preconditions**: Caller must satisfy preconditions for safe use
4. **Size-based deallocation**: `deallocate` takes size parameter (original uses Layout)

These differences are intentional design choices for verification clarity. The original's
`GlobalAlloc` implementation can be added as a thin wrapper around the verified core.

## Assumptions

1. **Alignment <= Size**: The `allocate(size)` API assumes the caller's alignment requirement
   is at most `size`. Blocks are naturally aligned to their block size (e.g., 64-byte blocks
   are 64-byte aligned). If a caller needs alignment greater than the block size (e.g., 128-byte
   alignment for an 8-byte allocation), this allocator does NOT guarantee that alignment.
   This matches the behavior of the original slab-based allocator.

## Unverified Components

The following components from the original `kheap.rs` are **not verified** due to Verus limitations
or because they are thin wrappers around verified logic:

### 1. Static Storage (`HEAP_STORAGE`, `HEAP`)

```rust
// Original (not verifiable in Verus)
static mut HEAP_STORAGE: HeapStorage = HeapStorage { memory: [0; MIN_HEAP_SIZE] };
static mut HEAP: Option<Kheap> = None;
```

**Reason**: Verus cannot verify mutable static state. The verified `init(addr, size)` function
takes explicit parameters instead of relying on static storage.

**Mitigation**: The caller must ensure the memory passed to `init()` is valid, writable,
and properly aligned. This is a precondition of the verified function.

### 2. ArenaAllocator and GlobalAlloc Trait

```rust
// Original (not verifiable in Verus)
#[global_allocator]
static mut ALLOCATOR: ArenaAllocator = ArenaAllocator;

unsafe impl GlobalAlloc for ArenaAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { ... }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { ... }
}
```

**Reason**: The `GlobalAlloc` trait is a thin wrapper that:
1. Accesses the global `HEAP` static
2. Extracts `size` from `Layout`
3. Calls the verified `Kheap::allocate`/`deallocate`

**Mitigation**: Since the wrapper only extracts `layout.size()` and calls verified functions,
the safety properties proven for `Kheap::allocate`/`deallocate` apply. A trusted wrapper
can be added that calls the verified core.

### 3. Layout Type

```rust
// Original uses std::alloc::Layout
unsafe fn allocate(&mut self, layout: Layout) -> Result<*mut u8, AllocError>
```

**Reason**: Verus does not have access to `std::alloc::Layout`. The verified version uses
`size: usize` directly, which is what `layout.size()` returns.

**Mitigation**: The verified `allocate(size)` is functionally equivalent to the original
`allocate(layout)` when `size = layout.size()`. Alignment is handled by natural block
alignment (see Assumptions section).

### Summary: What IS Verified

| Component | Verified? | Notes |
|-----------|-----------|-------|
| `Kheap::from_raw_parts` | ✅ Yes | Full verification with disjointness proof |
| `Kheap::allocate` | ✅ Yes | Full verification with postconditions |
| `Kheap::deallocate` | ✅ Yes | Full verification with frame conditions |
| `layout_to_slab_size` | ✅ Yes | Full verification |
| `init` | ✅ Yes | Wrapper around from_raw_parts |
| `SlabSize` enum | ✅ Yes | Verified spec_as_int and as_usize |
| `HEAP_STORAGE` | ❌ No | Static storage (Verus limitation) |
| `HEAP` global | ❌ No | Mutable static (Verus limitation) |
| `ArenaAllocator` | ❌ No | Thin wrapper (trusted) |
| `GlobalAlloc` trait | ❌ No | Thin wrapper (trusted) |

## Future Work

1. **Verify Slab integration**: Use the verified Slab from `../slab/` instead of trusted abstraction
2. **GlobalAlloc wrapper**: Add a trusted wrapper implementing the GlobalAlloc trait
3. **Thread safety**: Add specifications for concurrent access (requires ownership model)
