# Frame Allocator Verification: A Comprehensive Guide

This document provides a detailed introduction to the physical frame allocator,
its correctness specifications, and a guide to reading the Verus proof code.

## Table of Contents

1. [What is a Frame Allocator?](#1-what-is-a-frame-allocator)
2. [Core Correctness Specification](#2-core-correctness-specification)
3. [How Correctness is Achieved](#3-how-correctness-is-achieved)
4. [Guide to Reading the Proof Code](#4-guide-to-reading-the-proof-code)
5. [File Structure](#5-file-structure)
6. [Key Verus Concepts](#6-key-verus-concepts)

---

## 1. What is a Frame Allocator?

### 1.1 Purpose

The **frame allocator** manages physical memory at the granularity of **frames**
(4KB pages). In an operating system kernel, physical memory is divided into
fixed-size chunks called frames, and the frame allocator tracks which frames
are in use and which are free.

Unlike a heap allocator (which handles variable-sized allocations), a frame
allocator:

- **Fixed-size blocks**: Always allocates exactly 4096 bytes (one frame)
- **Physical addresses**: Works with physical memory addresses
- **Bitmap-based**: Uses a bitmap for O(1) allocation tracking
- **Foundation for virtual memory**: Provides backing frames for page tables

### 1.2 Architecture

The frame allocator uses a **bitmap** where each bit represents one frame:

```
                     Frame Allocator Memory Model
┌─────────────────────────────────────────────────────────────────────┐
│                        Physical Memory                               │
├─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬────────┤
│ Frame 0 │ Frame 1 │ Frame 2 │ Frame 3 │ Frame 4 │ Frame 5 │  ...   │
│  4 KB   │  4 KB   │  4 KB   │  4 KB   │  4 KB   │  4 KB   │        │
├─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴────────┤
│ 0x0000  │ 0x1000  │ 0x2000  │ 0x3000  │ 0x4000  │ 0x5000  │  ...   │
└─────────────────────────────────────────────────────────────────────┘

                         Bitmap Representation
                    ┌───┬───┬───┬───┬───┬───┬───┬───┐
                    │ 1 │ 0 │ 1 │ 1 │ 0 │ 0 │ 0 │...│
                    └───┴───┴───┴───┴───┴───┴───┴───┘
                      ↑   ↑   ↑   ↑   ↑   ↑   ↑
                      │   │   │   │   │   │   └── Frame 6: Free
                      │   │   │   │   │   └────── Frame 5: Free
                      │   │   │   │   └────────── Frame 4: Free
                      │   │   │   └────────────── Frame 3: Allocated
                      │   │   └────────────────── Frame 2: Allocated
                      │   └────────────────────── Frame 1: Free
                      └────────────────────────── Frame 0: Allocated

Bit set (1) = Frame is allocated
Bit unset (0) = Frame is free
```

### 1.3 Key Constants

```rust
FRAME_SIZE = 4096              // 4 KB per frame
MAX_FRAME_NUMBER = 0xFFFF_FFFF / FRAME_SIZE  // ~1 million frames (32-bit)
```

### 1.4 Core Operations

| Operation | Description |
|-----------|-------------|
| `alloc()` | Find and allocate a free frame, return its index |
| `alloc_address()` | Allocate and return a `FrameAddress` |
| `free(frame)` | Mark a frame as free |
| `book(addr)` | Reserve a specific frame (for memory-mapped I/O, etc.) |
| `alloc_range(start, count)` | Allocate a contiguous range of frames |

---

## 2. Core Correctness Specification

The correctness of the frame allocator rests on **memory safety** and
**ownership tracking**. We must prove that allocated frames never overlap
and that ownership is properly tracked.

### 2.1 The Central Safety Property: No Memory Aliasing

**Theorem (Frame Disjointness)**: All allocated frames occupy disjoint memory regions.

```
∀ i, j ∈ allocated_frames, i ≠ j:
    [i × FRAME_SIZE, (i+1) × FRAME_SIZE) ∩ [j × FRAME_SIZE, (j+1) × FRAME_SIZE) = ∅
```

This is trivially true because frames are at fixed offsets and non-overlapping by construction.

```rust
pub open spec fn frames_are_disjoint(i: int, j: int) -> bool {
    let addr_i = i * FRAME_SIZE as int;
    let addr_j = j * FRAME_SIZE as int;
    // Frames are disjoint if their memory regions don't overlap
    addr_i + FRAME_SIZE as int <= addr_j || addr_j + FRAME_SIZE as int <= addr_i
}
```

**Key Insight**: For any `i ≠ j`, we have `|i - j| ≥ 1`, which means
`|i × FRAME_SIZE - j × FRAME_SIZE| ≥ FRAME_SIZE`, guaranteeing disjointness.

### 2.2 Why Disjointness Matters

Without disjointness, the following disaster could occur:

```rust
// Hypothetical bug scenario (cannot happen with our proof)
frame1 = allocator.alloc();   // Returns frame 5
frame2 = allocator.alloc();   // BUG: Also returns frame 5!
write_to_frame(frame1, data1);  // Writes to physical address 0x5000
write_to_frame(frame2, data2);  // Overwrites frame1's data!
```

Our disjointness proof guarantees this CANNOT happen.

### 2.3 The Invariant

The allocator invariant (`FrameAllocator::inv()`) captures all correctness properties:

```rust
pub closed spec fn inv(&self) -> bool {
    // 1. Underlying bitmap is valid
    &&& self.bitmap.inv()
    
    // 2. Capacity is positive and bounded
    &&& self.bitmap@.number_of_bits() > 0
    &&& self.bitmap@.number_of_bits() <= MAX_FRAME_NUMBER as int + 1
    
    // 3. View matches bitmap state
    &&& self@.capacity == self.bitmap@.number_of_bits()
    
    // 4. Allocation state matches bitmap bits
    &&& forall|i: int| 0 <= i < self.bitmap@.number_of_bits() ==>
        (self@.is_allocated(i) <==> self.bitmap.is_bit_set(i))
    
    // 5. CRITICAL: No memory aliasing (first-class invariant)
    &&& self@.no_memory_aliasing()
    
    // 6. All allocated frames are in valid range
    &&& self@.allocated_frames_in_range()
}
```

### 2.4 Ghost Ownership Tracking

To prevent use-after-free and double-free bugs, we use **ghost permissions**:

```rust
pub tracked struct FramePermission {
    pub ghost frame_idx: int,      // Which frame this permission is for
    pub ghost allocator_id: int,   // Which allocator issued this permission
}
```

**Ownership Rules**:
1. `alloc_tracked()` creates a new `FramePermission` for the allocated frame
2. `free_tracked()` consumes the permission (linear type semantics)
3. Cross-allocator confusion is prevented by checking `allocator_id`

### 2.5 Allocation Postconditions

When `alloc()` succeeds, we guarantee:

```rust
ensures
    // 1. Invariant preserved
    self.inv(),
    
    // 2. LIVENESS: If free frames exist, allocation succeeds
    old(self)@.has_free_frame() ==> result is Ok,
    
    // 3. Returned frame was free, is now allocated
    result is Ok ==> {
        let frame_idx = result->Ok_0 as int;
        &&& 0 <= frame_idx < self@.capacity
        &&& self@.is_allocated(frame_idx)
        &&& !old(self)@.is_allocated(frame_idx)
    },
    
    // 4. Frame condition: other frames unchanged
    result is Ok ==> forall|i: int| i != frame_idx ==>
        self@.is_allocated(i) == old(self)@.is_allocated(i),
    
    // 5. COUNT: allocated count increases by 1
    result is Ok ==> 
        self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
    
    // 6. On error, state unchanged
    result is Err ==> self@ == old(self)@,
```

### 2.6 Deallocation Postconditions

When `free_tracked()` succeeds:

```rust
ensures
    // 1. Invariant preserved
    self.inv(),
    
    // 2. Frame is now free
    !self@.is_allocated(frame.spec_frame_number()),
    
    // 3. Other frames unchanged
    forall|i: int| i != frame.spec_frame_number() ==>
        self@.is_allocated(i) == old(self)@.is_allocated(i),
    
    // 4. COUNT: allocated count decreases by 1
    self.spec_num_allocated() == old(self).spec_num_allocated() - 1,
```

### 2.7 Range Allocation Postconditions

When `alloc_range(start, count)` succeeds:

```rust
ensures
    // 1. Invariant preserved
    self.inv(),
    
    // 2. LIVENESS: Always succeeds when preconditions met
    result is Ok,
    
    // 3. All frames in range are allocated
    forall|i: int| start <= i < start + count ==> self@.is_allocated(i),
    
    // 4. Frames outside range unchanged
    forall|i: int| !(start <= i < start + count) ==>
        self@.is_allocated(i) == old(self)@.is_allocated(i),
    
    // 5. COUNT: allocated count increases by count
    self.spec_num_allocated() == old(self).spec_num_allocated() + count,
```

---

## 3. How Correctness is Achieved

### 3.1 The Bitmap Abstraction

The bitmap is a **trusted dependency** (verified separately). Its spec provides:

```rust
// Allocation finds a free bit and sets it
fn alloc(&mut self) -> Result<usize, Error>
    ensures
        result is Ok ==> {
            let idx = result->Ok_0;
            &&& self.is_bit_set(idx as int)      // Bit is now set
            &&& !old(self).is_bit_set(idx as int) // Was previously unset
        },

// Set always succeeds when index is valid
fn set(&mut self, index: usize) -> Result<(), Error>
    requires (index as int) < self@.number_of_bits(),
    ensures result is Ok,  // Liveness guarantee
```

### 3.2 The View Pattern

We use a **ghost view** to abstract the concrete state:

```rust
pub ghost struct FrameAllocatorView {
    pub allocated_frames: Set<int>,  // Set of allocated frame indices
    pub capacity: int,               // Total number of frames
}

impl View for FrameAllocator {
    type V = FrameAllocatorView;
    
    closed spec fn view(&self) -> FrameAllocatorView {
        FrameAllocatorView {
            allocated_frames: self.bitmap@.bits.filter(|b| b).indices(),
            capacity: self.bitmap@.number_of_bits(),
        }
    }
}
```

### 3.3 Connecting Bitmap to View

A key lemma connects the concrete bitmap state to the abstract view:

```rust
proof fn lemma_allocated_iff_bit_set(&self, i: int)
    requires
        self.inv(),
        0 <= i < self@.capacity,
    ensures
        self@.is_allocated(i) <==> self.bitmap.is_bit_set(i),
```

This lemma is essential because:
- The **invariant is closed** (internal details hidden)
- Proofs need to know the bitmap-allocation correspondence
- The lemma **reveals** this connection when needed

### 3.4 Why Frame Disjointness is Automatic

The disjointness proof is straightforward because frames are at fixed offsets:

```rust
/// Lemma: Any two distinct frame indices have disjoint memory regions.
proof fn lemma_frames_disjoint(i: int, j: int)
    requires i != j,
    ensures FrameAllocatorView::frames_are_disjoint(i, j),
{
    // Key insight: |i - j| >= 1
    // Therefore: |i * FRAME_SIZE - j * FRAME_SIZE| >= FRAME_SIZE
    // This means the frames don't overlap
    
    if i < j {
        assert(i + 1 <= j);
        assert((i + 1) * FRAME_SIZE as int <= j * FRAME_SIZE as int);
        assert(i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int);
    } else {
        assert(j + 1 <= i);
        assert((j + 1) * FRAME_SIZE as int <= i * FRAME_SIZE as int);
        assert(j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int);
    }
}
```

### 3.5 No Memory Aliasing as First-Class Invariant

Unlike a separate lemma, `no_memory_aliasing()` is **part of the invariant**:

```rust
pub open spec fn no_memory_aliasing(&self) -> bool {
    forall|i: int, j: int|
        (self.is_allocated(i) && self.is_allocated(j) && i != j) ==>
        self.frames_are_disjoint(i, j)
}

pub closed spec fn inv(&self) -> bool {
    // ...other conditions...
    &&& self@.no_memory_aliasing()  // Memory safety is first-class
}
```

This means **every operation automatically preserves memory safety** by
preserving the invariant, without needing separate preservation lemmas.

### 3.6 Ghost Permission Flow

The ownership tracking uses linear ghost types:

```
   ┌──────────────────────────────────────────────────────────────────┐
   │                    Permission Lifecycle                          │
   └──────────────────────────────────────────────────────────────────┘
   
   alloc_tracked()                              free_tracked(perm)
        │                                              │
        ▼                                              ▼
   ┌─────────────┐                              ┌─────────────┐
   │ Permission  │───────── owned by ──────────▶│ Permission  │
   │   Created   │          caller              │  Consumed   │
   └─────────────┘                              └─────────────┘
        │                                              │
        │  perm.frame_idx = allocated_frame            │  Must match:
        │  perm.allocator_id = this_allocator          │  - frame being freed
        │                                              │  - this allocator
        │                                              │
        ▼                                              ▼
   Caller can prove                             Cannot free again
   exclusive access                             (permission is gone)
```

---

## 4. Guide to Reading the Proof Code

### 4.1 Verus Syntax Primer

Verus extends Rust with verification constructs:

```rust
// Specification function (ghost code, not executed)
pub open spec fn is_valid_frame(idx: int, capacity: int) -> bool {
    0 <= idx < capacity
}

// Executable function with pre/postconditions
fn allocate_frame(&mut self) -> (result: Result<usize, Error>)
    requires        // Preconditions (caller must satisfy)
        self.inv(),
    ensures         // Postconditions (function guarantees)
        self.inv(),
        result is Ok ==> result->Ok_0 < self@.capacity,
{
    self.bitmap.alloc()
}

// Proof function (proves a lemma)
proof fn lemma_frames_disjoint(i: int, j: int)
    requires i != j,
    ensures frames_are_disjoint(i, j),
{
    // Proof body
}
```

### 4.2 Reading Order

To understand the proof, read in this order:

#### Step 1: Understand the Types (frame_address.rs, lines 1-150)

```rust
// Frame number (index into physical memory)
pub struct FrameNumber(usize);

// Frame address (physical address, always frame-aligned)
pub struct FrameAddress(usize);

// Page-aligned physical address
pub struct PageAlignedPhysAddr(usize);

// Key conversions:
// FrameNumber(n) ←→ FrameAddress(n * FRAME_SIZE)
// PageAlignedPhysAddr(addr) → FrameNumber(addr / FRAME_SIZE)
```

#### Step 2: Understand the Permission Type (permission.rs, lines 40-130)

```rust
// Ghost permission proving ownership of a frame
pub tracked struct FramePermission {
    pub ghost frame_idx: int,
    pub ghost allocator_id: int,
}

// Permission set for range allocations
pub ghost struct FramePermissionSet {
    pub permissions: Set<FramePermission>,
}
```

#### Step 3: Understand the View (frame_core.rs, lines 60-150)

```rust
pub ghost struct FrameAllocatorView {
    pub allocated_frames: Set<int>,
    pub capacity: int,
}

impl FrameAllocatorView {
    // Is a specific frame allocated?
    pub open spec fn is_allocated(&self, frame_idx: int) -> bool {
        self.allocated_frames.contains(frame_idx)
    }
    
    // Do two frames have disjoint memory?
    pub open spec fn frames_are_disjoint(i: int, j: int) -> bool
    
    // Are all allocated frames disjoint?
    pub open spec fn no_memory_aliasing(&self) -> bool
    
    // Is there at least one free frame?
    pub open spec fn has_free_frame(&self) -> bool
}
```

#### Step 4: Understand the Invariant (frame_core.rs, lines 180-220)

```rust
pub closed spec fn inv(&self) -> bool {
    &&& self.bitmap.inv()                              // Bitmap valid
    &&& capacity > 0 && capacity <= MAX + 1            // Bounds
    &&& self@.capacity == bitmap.number_of_bits()      // Consistency
    &&& forall|i| is_allocated(i) <==> is_bit_set(i)   // Bitmap correspondence
    &&& self@.no_memory_aliasing()                     // MEMORY SAFETY
    &&& self@.allocated_frames_in_range()              // Bounds safety
}
```

This is the HEART of the verification. Every operation must preserve this.

#### Step 5: Read the Core Functions

**`new`** (lines 250-320):
- Creates allocator with empty bitmap
- Establishes initial invariant (no frames allocated)

**`alloc`** (lines 380-480):
- Delegates to bitmap.alloc()
- Proves liveness: `has_free_frame() ==> Ok`
- Proves frame condition

**`alloc_tracked`** (lines 520-600):
- Calls `alloc`
- Creates ghost permission
- Returns both frame index and permission

**`free_tracked`** (lines 775-850):
- Requires matching permission
- Consumes permission (cannot be used again)
- Marks frame as free

**`book`** (lines 845-880):
- Reserves specific frame
- Idempotent (can book already-allocated frames)
- Liveness: always succeeds when in range

**`alloc_range`** (lines 890-990):
- Loop-based range allocation
- Full liveness guarantee
- Count postcondition

**`alloc_range_inner`** (lines 1000-1070):
- Helper with loop invariants
- Tracks count through loop iterations

#### Step 6: Read the Lemmas (lines 1100-1200)

```rust
// Frames at different indices don't overlap
proof fn lemma_frames_disjoint(i: int, j: int)

// Reveal bitmap-allocation connection
proof fn lemma_allocated_iff_bit_set(&self, i: int)

// Liveness connection
proof fn lemma_has_free_frame_implies_bitmap_has_free_bit(&self)

// Fresh allocator has no aliasing (vacuously true)
proof fn lemma_fresh_allocator_no_aliasing(alloc: &FrameAllocator)

// Allocation preserves no-aliasing
proof fn lemma_alloc_preserves_no_aliasing(...)

// Deallocation preserves no-aliasing
proof fn lemma_dealloc_preserves_no_aliasing(...)
```

### 4.3 Key Patterns to Recognize

#### Pattern 1: The `@` Operator (View)

```rust
self@               // Gets FrameAllocatorView
self.bitmap@        // Gets BitmapView
self.allocator_id@  // Gets the int inside Ghost<int>
```

#### Pattern 2: Ghost Fields

```rust
pub struct FrameAllocator {
    bitmap: Bitmap,             // Real field (exists at runtime)
    allocator_id: Ghost<int>,   // Ghost field (only for verification)
}

// Initialize ghost field
allocator_id: Ghost(unique_id),

// Access ghost field in spec
self.allocator_id@
```

#### Pattern 3: Tracked Permissions

```rust
// Creating a permission (in proof code)
proof fn new(frame_idx: int, allocator_id: int) -> (tracked result: FramePermission)

// Receiving a permission (in exec code)
pub fn free_tracked(&mut self, frame: FrameAddress, Tracked(perm): Tracked<FramePermission>)
    requires
        perm.frame_idx == frame.spec_frame_number(),
        perm.allocator_id == old(self).spec_allocator_id(),
```

#### Pattern 4: Proof Blocks in Exec Code

```rust
fn alloc(&mut self) -> Result<usize, Error> {
    match self.bitmap.alloc() {
        Ok(frame_idx) => {
            proof {
                // Help the verifier connect bitmap state to allocation state
                self.lemma_allocated_iff_bit_set(frame_idx as int);
            }
            Ok(frame_idx)
        }
        Err(e) => Err(e),
    }
}
```

#### Pattern 5: Loop Invariants

```rust
while idx < end_frame
    invariant
        self.inv(),                                          // Invariant preserved
        self.spec_allocator_id() == old(self).spec_allocator_id(), // ID unchanged
        start_frame <= idx <= end_frame,                     // Bounds
        forall|i| start <= i < idx ==> self.bitmap.is_bit_set(i),  // Progress
        forall|i| idx <= i < end ==> !self.bitmap.is_bit_set(i),   // Remaining
        self.spec_num_allocated() == old + (idx - start),    // Count tracking
    decreases end_frame - idx,                               // Termination
{
    let _ = self.bitmap.set(idx);
    idx = idx + 1;
}
```

### 4.4 Understanding the Permission System

The permission system enforces exclusive ownership:

```rust
// 1. Allocation creates a permission
pub fn alloc_tracked(&mut self) -> Result<(usize, Tracked<FramePermission>), Error> {
    match self.alloc() {
        Ok(frame_idx) => {
            proof {
                let tracked perm = FramePermission::new(
                    frame_idx as int,
                    self.allocator_id@,
                );
            }
            Ok((frame_idx, Tracked(perm)))
        }
        Err(e) => Err(e),
    }
}

// 2. Deallocation consumes the permission
pub fn free_tracked(&mut self, frame: FrameAddress, Tracked(perm): Tracked<FramePermission>)
    requires
        perm.frame_idx == frame.spec_frame_number(),  // Right frame
        perm.allocator_id == old(self).spec_allocator_id(),  // Right allocator
    // ... perm is consumed, cannot be used again
```

The permission is a **linear type**: it must be used exactly once (to free),
and cannot be duplicated. This prevents:

- **Double-free**: Would need two permissions
- **Use-after-free by others**: Only permission holder can free
- **Cross-allocator confusion**: Allocator ID must match

---

## 5. File Structure

```
verus/frame/
├── lib.rs              # Crate entry point and re-exports
├── error.rs            # Error types (ErrorCode, Error)
├── bitmap.rs           # Bitmap abstraction (external_body, trusted)
├── frame_address.rs    # Frame address types (FrameNumber, FrameAddress, etc.)
├── permission.rs       # Ghost ownership types (FramePermission)
├── frame_core.rs       # Main verified implementation (~1200 lines)
├── original_frame.rs   # Original source code for reference
├── README.md           # Overview documentation
├── VERIFICATION_SUMMARY.md  # Verification statistics
└── PROOF_GUIDE.md      # This file
```

### 5.1 frame_core.rs Structure

```
Lines 1-35:      Copyright, imports
Lines 36-60:     Constants (FRAME_SIZE, MAX_FRAME_NUMBER)
Lines 60-150:    FrameAllocatorView ghost struct and spec functions
Lines 150-220:   FrameAllocator struct and inv()
Lines 220-280:   View implementation
Lines 280-380:   new() and from_raw_storage()
Lines 380-520:   alloc(), alloc_address()
Lines 520-610:   alloc_tracked(), alloc_address_tracked()
Lines 610-700:   free_internal() (private helper)
Lines 700-770:   free_untracked() (private)
Lines 770-850:   free_tracked() (public, consumes permission)
Lines 850-890:   book() with liveness
Lines 890-990:   alloc_range() with loop
Lines 990-1070:  alloc_range_inner() with loop invariants
Lines 1070-1170: Memory safety lemmas
Lines 1170-1220: Test proofs
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
fn alloc(&mut self) -> (result: Result<usize, Error>)
    requires    // PRECONDITIONS (caller must satisfy)
        self.inv(),
    ensures     // POSTCONDITIONS (function guarantees)
        self.inv(),
        result is Ok ==> self@.is_allocated(result->Ok_0 as int),
```

### 6.3 Tracked Types

Tracked types exist only for verification but enforce linear (must-use-once) semantics:

```rust
pub tracked struct FramePermission {
    pub ghost frame_idx: int,
    pub ghost allocator_id: int,
}

// Tracked parameter is consumed
fn free_tracked(&mut self, Tracked(perm): Tracked<FramePermission>)
// After this call, perm cannot be used again
```

### 6.4 Ghost vs Tracked

| Type | Purpose | Semantics |
|------|---------|-----------|
| `Ghost<T>` | Specification data | Copy freely, no runtime effect |
| `Tracked<T>` | Ownership proofs | Linear (use exactly once) |

```rust
pub struct FrameAllocator {
    bitmap: Bitmap,             // Runtime data
    allocator_id: Ghost<int>,   // Ghost (for specs)
}

pub fn alloc_tracked(&mut self) -> Result<(usize, Tracked<FramePermission>), Error>
// Returns Tracked permission that must be used exactly once
```

### 6.5 The View Trait

```rust
impl View for FrameAllocator {
    type V = FrameAllocatorView;
    
    closed spec fn view(&self) -> FrameAllocatorView {
        // Map concrete state to abstract state
    }
}

// Usage: self@ returns the FrameAllocatorView
```

### 6.6 Closed vs Open Spec Functions

```rust
// Open: Body visible to callers (can reason about internals)
pub open spec fn is_allocated(&self, i: int) -> bool {
    self.allocated_frames.contains(i)
}

// Closed: Body hidden (callers see only signature)
pub closed spec fn inv(&self) -> bool {
    // Implementation details hidden
    // Use lemmas to reveal specific facts
}
```

Use `closed` for invariants to prevent proofs from depending on internal details,
enabling future refactoring without breaking client proofs.

---

## Summary

The frame allocator verification proves memory safety through:

1. **No-Aliasing Invariant**: All allocated frames have disjoint memory regions
   - This is a **first-class invariant**, not a derived lemma
   
2. **Automatic Disjointness**: Frames at different indices trivially don't overlap
   - `i ≠ j ⟹ |i - j| ≥ 1 ⟹ regions don't overlap`

3. **Ghost Ownership**: Tracked permissions prove exclusive access
   - Prevents double-free, use-after-free, cross-allocator bugs
   - Linear type semantics (use exactly once)

4. **Liveness Guarantees**: All operations succeed when preconditions are met
   - `alloc`: `has_free_frame() ==> Ok`
   - `book`: Always succeeds when in range
   - `alloc_range`: Always succeeds when all frames free

5. **Explicit Count Tracking**: All operations have count postconditions
   - `alloc`: `num_allocated() == old + 1`
   - `free`: `num_allocated() == old - 1`
   - `alloc_range`: `num_allocated() == old + count`

The proof achieves **43 verified items with 0 errors** and uses:
- **Zero `assume` or `admit`** in core frame allocator logic
- **Minimal `external_body`** (only for trusted Bitmap dependency)
- **First-class memory safety** via invariant inclusion

This provides a machine-checked guarantee that the frame allocator correctly
manages physical memory without the risk of allocating overlapping regions
or violating ownership invariants.
