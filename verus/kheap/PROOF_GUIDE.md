# Kheap Verification: A Comprehensive Guide

This document provides a detailed introduction to the kernel heap allocator (kheap),
its correctness specifications, and a guide to reading the Verus proof code.

## Table of Contents

1. [What is Kheap?](#1-what-is-kheap)
2. [Core Correctness Specification](#2-core-correctness-specification)
3. [How Correctness is Achieved](#3-how-correctness-is-achieved)
4. [Guide to Reading the Proof Code](#4-guide-to-reading-the-proof-code)
5. [File Structure](#5-file-structure)
6. [Key Verus Concepts](#6-key-verus-concepts)

---

## 1. What is Kheap?

### 1.1 Purpose

The **kernel heap allocator (kheap)** is a memory allocator used by the Nanvix operating
system kernel to dynamically allocate and free memory. Unlike user-space allocators,
a kernel heap must be:

- **Deterministic**: Predictable allocation times for real-time guarantees
- **Memory-safe**: No buffer overflows, use-after-free, or double-free bugs
- **Efficient**: Minimal fragmentation and fast allocation/deallocation

### 1.2 Architecture

Kheap uses a **multi-slab architecture** with 8 fixed-size slabs:

```
                         Kernel Heap Memory Layout
┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
│  Slab 8  │ Slab 16  │ Slab 32  │ Slab 64  │ Slab 128 │ Slab 256 │ Slab 512 │Slab 4096 │
│  8-byte  │ 16-byte  │ 32-byte  │ 64-byte  │ 128-byte │ 256-byte │ 512-byte │4096-byte │
│  blocks  │  blocks  │  blocks  │  blocks  │  blocks  │  blocks  │  blocks  │  blocks  │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
     ↑           ↑          ↑          ↑           ↑          ↑          ↑          ↑
   addr      addr+1×S    addr+2×S   addr+3×S    addr+4×S   addr+5×S   addr+6×S   addr+7×S

Where S = slab_size = total_size / 8
```

### 1.3 How Allocation Works

When a caller requests `n` bytes:

1. **Size Classification**: Map `n` to the smallest sufficient slab
   - 1-8 bytes → Slab8 (8-byte blocks)
   - 9-16 bytes → Slab16 (16-byte blocks)
   - 17-32 bytes → Slab32 (32-byte blocks)
   - ... and so on

2. **Slab Allocation**: Allocate a block from the selected slab

3. **Return Pointer**: Return a pointer to the allocated block

### 1.4 Key Design Constraint

**Size Gap**: Sizes 513-4095 bytes are NOT supported. The allocator jumps from 512-byte
blocks directly to 4096-byte blocks. This is a deliberate design choice in the original
implementation.

---

## 2. Core Correctness Specification

The correctness of kheap boils down to **memory safety**. We must prove that the
allocator never returns overlapping memory regions to different callers.

### 2.1 The Central Safety Property: Disjointness

**Theorem (Slab Disjointness)**: All 8 slabs manage completely disjoint memory regions.

```
∀ i, j ∈ {8, 16, 32, 64, 128, 256, 512, 4096}, i ≠ j:
    Slab_i.region ∩ Slab_j.region = ∅
```

This means:
- An address allocated from Slab8 can NEVER be in Slab16's region
- An address allocated from Slab32 can NEVER be in Slab64's region
- ... for all 28 pairs of slabs

### 2.2 Why Disjointness Matters

Without disjointness, the following disaster could occur:

```
// Hypothetical bug scenario (cannot happen with our proof)
ptr1 = allocate(8);   // Returns address 0x1000 from Slab8
ptr2 = allocate(16);  // BUG: Also returns 0x1000 from Slab16!
*ptr1 = 42;           // Writes to 0x1000
*ptr2 = 99;           // Overwrites ptr1's data!
```

Our disjointness proof guarantees this CANNOT happen.

### 2.3 The Heap Invariant

The heap invariant (`Kheap::inv()`) captures all correctness properties:

```rust
pub closed spec fn inv(&self) -> bool {
    // 1. All 8 slabs are individually valid
    &&& self.slab_8_bytes.inv()
    &&& self.slab_16_bytes.inv()
    // ... (all 8 slabs)
    
    // 2. Each slab has the correct block size
    &&& self.slab_8_bytes@.block_size == 8
    &&& self.slab_16_bytes@.block_size == 16
    // ... (all 8 slabs)
    
    // 3. CRITICAL: All slabs are disjoint
    &&& self@.all_slabs_disjoint()
    
    // 4. All slabs are within the heap buffer
    &&& self@.all_slabs_within_extent()
    
    // 5. All slabs are properly aligned
    &&& self@.all_slabs_aligned()
    
    // 6. Heap extent is valid
    &&& self.base_addr@ > 0
    &&& self.total_size@ > 0
}
```

### 2.4 Allocation Postconditions

When `allocate(size)` succeeds, we guarantee:

```rust
ensures
    // 1. Heap invariant preserved
    self.inv(),
    
    // 2. Address is valid in the heap
    self@.is_valid_heap_addr(addr),
    
    // 3. Address is in the CORRECT slab (not any other)
    self@.get_slab(slab_size).is_valid_addr(addr),
    
    // 4. Block is marked as allocated
    self@.get_slab(slab_size).is_allocated(block_idx),
    
    // 5. Block size is sufficient
    slab_size.spec_as_int() >= size,
    
    // 6. Address is properly aligned
    addr % slab_size.spec_as_int() == 0,
    
    // 7. Frame condition: other slabs unchanged
    // (slab_16 unchanged if we allocated from slab_8, etc.)
```

### 2.5 Deallocation Postconditions

When `deallocate(ptr, size)` succeeds:

```rust
ensures
    // 1. Heap invariant preserved
    self.inv(),
    
    // 2. Block is freed in the correct slab
    !self@.get_slab(slab_size).is_allocated(block_idx),
    
    // 3. Frame condition: other slabs unchanged
```

---

## 3. How Correctness is Achieved

### 3.1 Construction-Time Proof

The key insight is that **disjointness is established at construction time** in
`from_raw_parts`. Once proven at construction, operations only modify allocation
state within slabs, never the memory regions themselves.

```rust
pub unsafe fn from_raw_parts(addr: usize, size: usize) -> Result<Kheap, Error>
```

The construction proof works by:

1. **Computing slab offsets**: `slab_i` starts at `addr + i * slab_size`
2. **Proving non-overlap**: For each pair (i, j), prove their regions don't overlap
3. **Establishing invariant**: All 8 slabs satisfy `inv()` with correct block sizes

### 3.2 The Disjointness Proof Structure

For two slabs to be disjoint, their memory regions must not overlap:

```rust
pub open spec fn slabs_disjoint(&self, s1: &SlabView, s2: &SlabView) -> bool {
    let s1_start = s1.data_addr;
    let s1_end = s1.data_addr + s1.num_data_blocks * s1.block_size;
    let s2_start = s2.data_addr;
    let s2_end = s2.data_addr + s2.num_data_blocks * s2.block_size;
    
    // Either s1 ends before s2 starts, or s2 ends before s1 starts
    s1_end <= s2_start || s2_end <= s1_start
}
```

We prove this for all 28 pairs:

```rust
pub open spec fn all_slabs_disjoint(&self) -> bool {
    // 7 pairs involving slab_8
    &&& self.slabs_disjoint(&self.slab_8, &self.slab_16)
    &&& self.slabs_disjoint(&self.slab_8, &self.slab_32)
    &&& self.slabs_disjoint(&self.slab_8, &self.slab_64)
    // ... all 28 pairs
}
```

### 3.3 Why the Proof Works

The memory layout guarantees disjointness:

```
Slab 0 (8-byte):    [addr + 0×S,  addr + 1×S)
Slab 1 (16-byte):   [addr + 1×S,  addr + 2×S)
Slab 2 (32-byte):   [addr + 2×S,  addr + 3×S)
...
Slab 7 (4096-byte): [addr + 7×S,  addr + 8×S)
```

Since each slab occupies `[addr + i×S, addr + (i+1)×S)`, and these intervals are
clearly non-overlapping (they're consecutive), disjointness follows mathematically.

### 3.4 Invariant Preservation

Operations preserve the invariant because:

1. **Allocation**: Only modifies `allocated_blocks` set within ONE slab
   - Memory regions don't change → disjointness preserved
   - Other slabs completely unchanged (frame condition)

2. **Deallocation**: Only modifies `allocated_blocks` set within ONE slab
   - Same reasoning as allocation

---

## 4. Guide to Reading the Proof Code

### 4.1 Verus Syntax Primer

Verus extends Rust with verification constructs:

```rust
// Specification function (ghost code, not executed)
pub open spec fn my_spec(x: int) -> bool {
    x > 0
}

// Executable function with pre/postconditions
fn my_function(x: usize) -> (result: usize)
    requires        // Preconditions (caller must satisfy)
        x > 0,
    ensures         // Postconditions (function guarantees)
        result == x * 2,
{
    x * 2
}

// Proof function (proves a lemma)
proof fn my_lemma(x: int)
    requires x > 0,
    ensures x + 1 > 1,
{
    // Proof body (often empty if trivial)
}
```

### 4.2 Reading Order

To understand the proof, read in this order:

#### Step 1: Understand the Data Types (lines 60-230)

```rust
// 1. SlabSize enum - the 8 slab categories
pub enum SlabSize {
    Slab8, Slab16, Slab32, Slab64, Slab128, Slab256, Slab512, Slab4096
}

// 2. KheapView - ghost representation of heap state
pub ghost struct KheapView {
    pub slab_8: SlabView,
    pub slab_16: SlabView,
    // ... all 8 slabs
    pub base_addr: int,
    pub total_size: int,
}

// 3. Kheap - the actual heap structure
pub struct Kheap {
    slab_8_bytes: Slab,
    slab_16_bytes: Slab,
    // ... all 8 slabs
    base_addr: Ghost<int>,   // Ghost field for verification
    total_size: Ghost<int>,
}
```

#### Step 2: Understand the Specifications (lines 140-300)

```rust
// Key spec functions in KheapView:

// Check if two slabs have disjoint memory regions
pub open spec fn slabs_disjoint(&self, s1: &SlabView, s2: &SlabView) -> bool

// Check if ALL 28 pairs are disjoint
pub open spec fn all_slabs_disjoint(&self) -> bool

// Check if an address is valid in the heap
pub open spec fn is_valid_heap_addr(&self, addr: int) -> bool

// Get the slab view for a given size category
pub open spec fn get_slab(&self, size: SlabSize) -> SlabView
```

#### Step 3: Understand the Invariant (lines 454-482)

```rust
pub closed spec fn inv(&self) -> bool {
    // All slabs valid + correct block sizes + disjoint + within extent + aligned
}
```

This is the HEART of the verification. Every operation must preserve this.

#### Step 4: Read the Core Functions

**`from_raw_parts`** (lines 515-800):
- Creates the heap
- PROVES disjointness for all 28 pairs
- PROVES all slabs are within extent
- PROVES all slabs are aligned

**`allocate`** (lines 885-975):
- Selects the correct slab
- Allocates from that slab
- PROVES the postconditions

**`deallocate`** (lines 1002-1058):
- Selects the correct slab
- Deallocates from that slab
- PROVES the frame condition

#### Step 5: Read the Lemmas (lines 1100-1250)

Lemmas are helper proofs that establish useful facts:

```rust
// If heap inv holds, all individual slab invs hold
proof fn lemma_inv_implies_slab_invs(&self)

// An address valid in one slab is NOT valid in any other
proof fn lemma_slabs_handle_disjoint_addresses(&self, addr: int)

// A fresh heap can allocate (liveness property)
proof fn lemma_fresh_heap_can_allocate(heap: &Kheap, size: int)
```

#### Step 6: Read the Tests (lines 1250-1850)

Verified tests serve as executable documentation:

```rust
// Test that size mapping is correct
fn test_layout_to_slab_size_verified()

// Test that disjointness implies address exclusivity
proof fn test_address_exclusivity_verified(view: KheapView, addr: int)
```

### 4.3 Key Patterns to Recognize

#### Pattern 1: The `@` Operator (View)

```rust
self.slab_8_bytes@  // Gets the ghost view (SlabView) of the Slab
self@               // Gets the ghost view (KheapView) of the Kheap
```

#### Pattern 2: Ghost Fields

```rust
pub struct Kheap {
    // Real fields (exist at runtime)
    slab_8_bytes: Slab,
    
    // Ghost fields (only exist in proofs)
    base_addr: Ghost<int>,
}

// Initialize ghost field
base_addr: Ghost(addr as int),

// Access ghost field in spec
self.base_addr@  // Returns the int value
```

#### Pattern 3: Proof Blocks in Exec Code

```rust
fn allocate(&mut self, size: usize) -> Result<*mut u8, Error> {
    // ... exec code ...
    
    proof {
        // Ghost code to help the verifier
        assert(self.slab_8_bytes@.is_valid_addr(addr));
    }
    
    // ... more exec code ...
}
```

#### Pattern 4: Match with Proof

```rust
match slab_size {
    SlabSize::Slab8 => {
        proof { assert(self.slab_8_bytes@.is_valid_addr(addr)); }
        self.slab_8_bytes.allocate()
    }
    // ...
}
```

#### Pattern 5: Ensures with Complex Expressions

```rust
ensures
    result is Ok ==> ({
        let heap = result->Ok_0;
        &&& heap.inv()
        &&& heap@.is_empty()
    }),
```

The `({...})` syntax groups multiple conditions in the postcondition.

### 4.4 Understanding the Disjointness Proof

The most complex part is proving disjointness in `from_raw_parts`. Here's how to read it:

```rust
// After constructing all 8 slabs, we prove disjointness:
proof {
    // Slab addresses for reference
    let s8_addr = addr as int;
    let s16_addr = addr as int + slab_size as int;
    // ...
    
    // Prove each pair is disjoint
    // Pair (8, 16): slab_8 ends at s16_addr, slab_16 starts at s16_addr
    assert(s8_end <= s16_start);  // Therefore disjoint
    
    // ... 27 more pairs ...
    
    // Finally, assert the full property
    assert(result@.all_slabs_disjoint());
}
```

---

## 5. File Structure

```
verus/kheap/
├── lib.rs              # Crate entry point
├── error.rs            # Error types (simple, trusted)
├── slab.rs             # Slab abstraction (external_body, verified in ../slab/)
├── kheap_core.rs       # Main verified implementation (~1800 lines)
├── README.md           # Overview documentation
├── VERIFICATION_SUMMARY.md  # Verification statistics
└── PROOF_GUIDE.md      # This file
```

### 5.1 kheap_core.rs Structure

```
Lines 1-35:      Copyright, imports
Lines 36-60:     Constants (NUM_OF_SLABS, MIN_HEAP_SIZE, etc.)
Lines 60-130:    SlabSize enum with spec_as_int, as_usize
Lines 130-230:   KheapView ghost struct and spec functions
Lines 230-300:   layout_to_slab_size function
Lines 300-400:   Lemmas for layout_to_slab_size
Lines 400-450:   Kheap struct definition
Lines 450-520:   Kheap::inv() and View implementation
Lines 520-850:   Kheap::from_raw_parts (construction + disjointness proof)
Lines 850-980:   Kheap::allocate
Lines 980-1060:  Kheap::deallocate
Lines 1060-1100: init() standalone function
Lines 1100-1250: Kheap lemmas
Lines 1250-1450: Core functionality tests
Lines 1450-1550: Memory safety property lemmas
Lines 1550-1850: Behavioral tests
```

---

## 6. Key Verus Concepts

### 6.1 Spec vs Exec vs Proof

| Mode | Runs at Runtime? | Can Call | Purpose |
|------|------------------|----------|---------|
| `spec fn` | No | spec | Define specifications |
| `fn` (exec) | Yes | exec, (proof in proof blocks) | Actual implementation |
| `proof fn` | No | spec, proof | Prove lemmas |

### 6.2 requires / ensures

```rust
fn foo(x: usize) -> (result: usize)
    requires    // PRECONDITIONS
        x > 0,  // Caller must satisfy this
    ensures     // POSTCONDITIONS
        result > x,  // Function guarantees this
```

### 6.3 Ghost State

Ghost state exists only for verification:

```rust
pub ghost struct KheapView { ... }  // Entire struct is ghost

pub struct Kheap {
    real_field: Slab,           // Exists at runtime
    ghost_field: Ghost<int>,    // Only for verification
}
```

### 6.4 The View Trait

```rust
impl View for Kheap {
    type V = KheapView;
    
    closed spec fn view(&self) -> KheapView {
        // Map concrete state to abstract state
    }
}

// Usage: self@ returns the KheapView
```

### 6.5 Closed vs Open Spec Functions

```rust
// Open: Body visible to callers (can be unfolded in proofs)
pub open spec fn is_valid(x: int) -> bool { x > 0 }

// Closed: Body hidden from callers (encapsulation)
pub closed spec fn inv(&self) -> bool { ... }
```

Use `closed` for invariants to prevent proofs from depending on internal details.

---

## Summary

The kheap verification proves memory safety through:

1. **Disjointness Invariant**: All 8 slabs manage non-overlapping memory regions
2. **Construction Proof**: Disjointness established when heap is created
3. **Preservation Proof**: Operations only modify allocation state, not regions
4. **Frame Conditions**: Each operation affects only the relevant slab

The proof achieves **43 verified items with 0 errors** and uses:
- **Zero `external_body`** in core kheap logic
- **Zero `assume` or `admit`** statements
- **Explicit enumeration** of all 28 disjoint pairs

This provides a machine-checked guarantee that the kernel heap allocator cannot
return overlapping memory to different allocation requests.
