# Nanvix Memory Management Formal Verification Roadmap

## Document Information

| Item | Description |
|------|-------------|
| Version | v2.0 |
| Last Updated | 2026-01-19 |
| Verification Tool | Verus |
| Target Module | Kernel Memory Management Subsystem |

---

## 1. Verification Methodology

### 1.1 Core Concepts: View, Invariant, and Refinement

In Verus, formal verification is based on the following core concepts:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       Refinement Verification Framework                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Abstract Spec Layer                Concrete Implementation Layer      │
│   ┌─────────────────┐               ┌─────────────────┐                │
│   │   SlabView      │◄───view()────│     Slab        │                │
│   │ ┌─────────────┐ │               │ ┌─────────────┐ │                │
│   │ │allocated:   │ │               │ │   bitmap    │ │                │
│   │ │  Set<int>   │ │               │ │   Bitmap    │ │                │
│   │ └─────────────┘ │               │ └─────────────┘ │                │
│   └─────────────────┘               └─────────────────┘                │
│           │                                  │                          │
│           │          inv() connects          │                          │
│           └──────────────────────────────────┘                          │
│                                                                         │
│   Refinement Proof = inv() preservation + ensures constrains View      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

| Concept | Verus Implementation | Purpose |
|---------|---------------------|---------|
| **Abstract State** | `View` type (e.g., `SlabView`) | Abstracts away implementation details, keeps only logical state |
| **Abstraction Function** | `view()` spec function | Maps concrete state to abstract state |
| **Coupling Invariant** | `inv()` spec function | Connects abstract and concrete, defines valid states |
| **Refinement** | `requires`/`ensures` | Proves implementation behavior conforms to abstract spec |

### 1.2 Verification Strategy: Property-Driven Bottom-Up

This project adopts a **property-driven bottom-up** strategy, combining the advantages of both approaches:

```
Traditional Top-Down       This Project's Approach      Traditional Bottom-Up
        │                          │                            │
        │                  ┌───────┴───────┐                    │
        │                  │ Define Top-Level│                   │
        │                  │ Safety Properties│                  │
        │                  └───────┬───────┘                    │
        │                          │                            │
        │                  ┌───────▼───────┐                    │
        │                  │ Decompose to   │                    │
        │                  │ Module Properties│                  │
        │                  └───────┬───────┘                    │
        │                          │                            │
        ▼                  ┌───────▼───────┐                    ▼
    High-Level ────────────│ Bottom-Up      │──────────── Low-Level
                           │ Verify Existing │
                           │ Code            │
                           └───────┬───────┘
                                   │
                           ┌───────▼───────┐
                           │ Compose Proofs │
                           │ for Top-Level  │
                           └───────────────┘
```

**Why this approach?**

1. Top-level properties guide verification direction from the start
2. Bottom-up is suitable for incremental verification of existing code
3. Composition proofs at the end ensure top-level properties hold

---

## 2. Project Status Analysis

### 2.1 Project Characteristics

| Dimension | Nanvix |
|-----------|--------|
| Architecture | Microkernel |
| Memory Management Complexity | Moderate (Frame allocation, Kpool/Upool, Page tables) |
| Existing Verification | Slab and Bitmap verified (`verus/` directory) |
| Code Style | Clean layered structure, minimal unsafe |

### 2.2 Existing Verification Results

| Module | Verified Items | assume | external_body | Status |
|--------|---------------|--------|---------------|--------|
| Bitmap | - | 0 | 0 | ✅ Complete |
| Slab | 85 | 0 | 0 (core) | ✅ Complete |
| Kheap | 31 | 0 | 1 (constructor) | ✅ Complete |

### 2.3 Module Dependency Graph

```
                    ┌─────────────────────────────────────────┐
                    │      Top-Level Safety Properties (P1-P10)│
                    └─────────────────┬───────────────────────┘
                                      │ need to prove
          ┌───────────────────────────┼───────────────────────────┐
          ▼                           ▼                           ▼
    ┌───────────┐              ┌───────────┐              ┌───────────┐
    │   Vmem    │              │   Upool   │              │   Kpool   │
    │(Page Table)│             │(User Pool)│              │(Kernel Pool)│
    └─────┬─────┘              └─────┬─────┘              └─────┬─────┘
          │                          │                          │
          │                          ▼                          │
          │                   ┌─────────────┐                   │
          │                   │FrameAlloc   │◄──────────────────┘
          │                   │ (frame.rs)  │
          │                   └─────┬───────┘
          │                         │
          │                         ▼
          │                   ┌───────────┐
          └──────────────────►│  Bitmap   │◄─────┐
                              └───────────┘      │
                                    ▲            │
                              ┌─────┴─────┐      │
                              │   Slab    │──────┘
                              └───────────┘
                                    ▲
                              ┌─────┴─────┐
                              │   Kheap   │ (Independent subsystem)
                              └───────────┘
```

---

## 3. Top-Level Safety Properties

**These properties are the ultimate verification goals and guide all work from the beginning.**

### 3.1 Memory Safety Properties

These properties ensure basic allocator correctness:

```rust
// P1: No Double Allocation
// An allocated frame cannot be allocated again
proof fn no_double_allocation()
    ensures
        forall|allocator: FrameAllocator, frame: FrameAddress|
            allocator.inv() && allocator@.allocated_frames.contains(frame)
            ==> allocator.alloc() != Ok(frame)

// P2: No Use After Free
// A freed frame is removed from the allocated set
proof fn no_use_after_free()
    ensures
        forall|allocator: FrameAllocator, frame: FrameAddress|
            allocator.free(frame).is_ok()
            ==> !allocator@.allocated_frames.contains(frame)

// P3: No Double Free
// Only allocated frames can be freed
proof fn no_double_free()
    ensures
        forall|allocator: FrameAllocator, frame: FrameAddress|
            allocator.free(frame).is_ok()
            ==> old(allocator)@.allocated_frames.contains(frame)
```

**Localization to modules**: These properties directly correspond to `FrameAllocator`'s `ensures` clauses.

### 3.2 Address Space Isolation Properties

```rust
// P4: User/Kernel Space Disjoint
proof fn user_kernel_disjoint()
    ensures
        forall|addr: VirtualAddress|
            is_user_addr(addr) ==> !is_kernel_addr(addr)

// P5: User Space Cannot Access Kernel Physical Memory
proof fn user_cannot_access_kernel_memory()
    ensures
        forall|vmem: Vmem, user_vaddr: VirtualAddress|
            vmem.inv()
            && Vmem::is_user_addr(user_vaddr)
            && vmem.lookup(user_vaddr).is_some()
            ==> !is_kernel_frame(vmem.lookup(user_vaddr).unwrap())
```

**Localization to modules**: P4 is a pure spec function on address types; P5 requires `Vmem::inv()` to include this constraint.

### 3.3 Page Table Consistency Properties

```rust
// P6: Map Then Lookup
proof fn map_then_lookup()
    ensures
        forall|vmem: Vmem, vaddr: VirtualAddress, frame: FrameAddress|
            vmem.inv() && vmem.map(vaddr, frame).is_ok()
            ==> vmem.lookup(vaddr) == Some(frame)

// P7: Unmap Then Not Found
proof fn unmap_then_not_found()
    ensures
        forall|vmem: Vmem, vaddr: VirtualAddress|
            vmem.inv() && vmem.unmap(vaddr).is_ok()
            ==> vmem.lookup(vaddr).is_none()
```

**Localization to modules**: Directly corresponds to `Vmem::map` and `Vmem::unmap`'s `ensures`.

### 3.4 Resource Exhaustion Handling Properties

```rust
// P8: Allocator Liveness
proof fn allocator_liveness()
    ensures
        forall|allocator: FrameAllocator|
            allocator.inv() && allocator@.has_free_frames()
            ==> allocator.alloc().is_ok()

// P9: Failure Safety
// State unchanged on failure - key to Refinement
proof fn allocation_failure_safe()
    ensures
        forall|allocator: FrameAllocator|
            allocator.inv() && allocator.alloc().is_err()
            ==> allocator@ == old(allocator)@
```

### 3.5 Copy Safety Properties

```rust
// P10: User/Kernel Copy Bounds Check
proof fn copy_bounds_safety()
    ensures
        forall|vmem: Vmem, src: VirtualAddress, dst: VirtualAddress, size: usize|
            copy_from_user(vmem, dst, src, size).is_ok()
            ==> Vmem::is_user_region(src, size)
                && Vmem::is_kernel_region(dst, size)
```

### 3.6 Property to Module Mapping

| Property | Primary Module | Verification Method |
|----------|---------------|---------------------|
| P1 No Double Allocation | FrameAllocator | `alloc()` ensures |
| P2 No Use After Free | FrameAllocator | `free()` ensures |
| P3 No Double Free | FrameAllocator | `free()` requires |
| P4 Space Disjoint | Address Types | spec function definition |
| P5 User Cannot Access Kernel | Vmem | `inv()` includes constraint |
| P6 Map Consistency | Vmem, PageTable | `map()` ensures |
| P7 Unmap Consistency | Vmem, PageTable | `unmap()` ensures |
| P8 Allocator Liveness | FrameAllocator | `alloc()` ensures |
| P9 Failure Safety | All Allocators | `ensures self@ == old(self)@` |
| P10 Copy Safety | Vmem | `copy_*` requires |

---

## 4. Phased Verification Roadmap

### Phase 0: Specification Design and Infrastructure (1-2 weeks)

**Goal**: Establish verification framework, define all View types and top-level properties

#### 0.1 Create Verification Module Structure

```
src/kernel/src/mm/
├── mod.rs              # Existing code
├── specs/              # New: Specification definitions
│   ├── mod.rs
│   ├── properties.rs   # Top-level properties P1-P10
│   ├── frame_spec.rs   # FrameAllocatorView
│   ├── vmem_spec.rs    # VmemView
│   ├── kheap_spec.rs   # Reuse existing
│   └── address_spec.rs # Address type specs
└── proofs/             # New: Composition proofs
    ├── mod.rs
    ├── isolation.rs    # Isolation proofs
    └── allocator.rs    # Allocator correctness proofs
```

#### 0.2 Define Core View Types

```rust
// specs/frame_spec.rs
verus! {
    /// Abstract state: only keeps logically important information
    pub struct FrameAllocatorView {
        pub allocated_frames: Set<int>,  // Set of allocated frames
        pub capacity: int,               // Total capacity
    }

    impl FrameAllocatorView {
        /// Whether there are free frames
        pub open spec fn has_free_frames(&self) -> bool {
            self.allocated_frames.len() < self.capacity
        }

        /// Whether a frame is allocated
        pub open spec fn is_allocated(&self, frame_idx: int) -> bool {
            self.allocated_frames.contains(frame_idx)
        }
    }
}
```

#### 0.3 Define Abstract Operation Specifications

```rust
// specs/frame_spec.rs (continued)
verus! {
    /// Abstract alloc operation specification
    pub open spec fn abstract_alloc(s: FrameAllocatorView) -> (FrameAllocatorView, Option<int>) {
        if s.has_free_frames() {
            // Choose an unallocated frame
            let frame = choose|f: int| 0 <= f < s.capacity && !s.is_allocated(f);
            (FrameAllocatorView {
                allocated_frames: s.allocated_frames.insert(frame),
                ..s
            }, Some(frame))
        } else {
            (s, None)  // State unchanged on failure
        }
    }

    /// Abstract free operation specification
    pub open spec fn abstract_free(s: FrameAllocatorView, frame: int) -> (FrameAllocatorView, bool) {
        if s.is_allocated(frame) {
            (FrameAllocatorView {
                allocated_frames: s.allocated_frames.remove(frame),
                ..s
            }, true)
        } else {
            (s, false)  // Freeing unallocated frame fails
        }
    }
}
```

#### 0.4 Migrate Existing Verification

- Integrate `verus/slab/`, `verus/bitmap/`, `verus/kheap/` into unified build system
- Ensure existing verification can be depended upon by new modules

---

### Phase 1: Low-Level Allocator Verification (2-3 weeks)

**Goal**: Verify FrameAllocator, prove P1-P3, P8-P9

**Key Point**: Define `inv()` to connect abstract and concrete, prove Refinement

#### 1.1 FrameAllocator View and Invariant

```rust
// mm/phys/frame.rs (add Verus specs)
verus! {
    impl FrameAllocator {
        /// Abstract state
        pub closed spec fn view(&self) -> FrameAllocatorView {
            FrameAllocatorView {
                allocated_frames: self.bitmap@.ones(),  // Bits set to 1 in Bitmap
                capacity: self.bitmap.number_of_bits() as int,
            }
        }

        /// Invariant: connects abstract and concrete
        pub open spec fn inv(&self) -> bool {
            // 1. Underlying Bitmap satisfies its invariant
            &&& self.bitmap.inv()
            // 2. Abstract state consistent with concrete state (Coupling Invariant)
            &&& forall|i: int| 0 <= i < self@.capacity
                ==> (self@.is_allocated(i) <==> self.bitmap.is_bit_set(i))
            // 3. Capacity is valid
            &&& self@.capacity > 0
        }
    }
}
```

#### 1.2 Prove Refinement

```rust
verus! {
    impl FrameAllocator {
        /// alloc satisfies abstract specification
        pub fn alloc(&mut self) -> (result: Result<FrameAddress, Error>)
            requires
                old(self).inv(),
            ensures
                // Preserve invariant
                self.inv(),
                // Refinement: behavior conforms to abstract operation
                match result {
                    Ok(frame) => {
                        let idx = frame.into_frame_number() as int;
                        // P1: Returned frame was previously unallocated
                        &&& !old(self)@.is_allocated(idx)
                        // State change conforms to abstract spec
                        &&& self@.allocated_frames == old(self)@.allocated_frames.insert(idx)
                        // P8: Success when free frames available
                    },
                    Err(_) => {
                        // P9: State unchanged on failure
                        self@ == old(self)@
                    }
                }
        {
            // Existing implementation...
        }

        /// free satisfies abstract specification
        pub fn free(&mut self, frame: FrameAddress) -> (result: Result<(), Error>)
            requires
                old(self).inv(),
                // P3: Can only free allocated frames (precondition)
                old(self)@.is_allocated(frame.into_frame_number() as int),
            ensures
                self.inv(),
                result.is_ok(),
                // P2: Removed from set after free
                !self@.is_allocated(frame.into_frame_number() as int),
                // Refinement: Other frames unaffected
                forall|f: int| f != frame.into_frame_number() as int
                    ==> (self@.is_allocated(f) <==> old(self)@.is_allocated(f))
        {
            // Existing implementation...
        }
    }
}
```

#### 1.3 Handle alloc_range Rollback Issue

**Current Problem**: `alloc_range()` may leave partially allocated frames on failure

```rust
verus! {
    impl FrameAllocator {
        pub fn alloc_range(&mut self, region: &TruncatedMemoryRegion<PhysicalAddress>)
            -> (result: Result<(), Error>)
            requires
                old(self).inv(),
            ensures
                self.inv(),
                // Key: Atomicity - either all succeed or state unchanged
                result.is_ok() ==> {
                    // All frames are allocated
                    forall|i: int| region.contains_frame(i)
                        ==> self@.is_allocated(i)
                },
                result.is_err() ==> {
                    // State completely unchanged on failure (requires implementation fix)
                    self@ == old(self)@
                }
        {
            // Need to modify implementation: add rollback logic
        }
    }
}
```

**Recommendation**: Modify `alloc_range` implementation to rollback set bits on failure.

---

### Phase 2: Memory Pool Verification (2 weeks)

**Goal**: Verify Kpool and Upool, which wrap FrameAllocator or Slab

#### 2.1 Upool (based on FrameAllocator)

```rust
verus! {
    pub struct UpoolView {
        pub frame_allocator_view: FrameAllocatorView,
    }

    impl Upool {
        pub closed spec fn view(&self) -> UpoolView {
            UpoolView {
                frame_allocator_view: self.inner.borrow().frame_allocator@,
            }
        }

        pub open spec fn inv(&self) -> bool {
            self.inner.borrow().frame_allocator.inv()
        }
    }
}
```

#### 2.2 Kpool (based on Slab)

```rust
verus! {
    pub struct KpoolView {
        pub slab_view: SlabView,  // Reuse verified SlabView
    }

    impl Kpool {
        pub closed spec fn view(&self) -> KpoolView {
            KpoolView {
                slab_view: self.inner.borrow().slab@,
            }
        }

        pub open spec fn inv(&self) -> bool {
            &&& self.inner.borrow().slab.inv()
            // Additional constraint: address alignment
            &&& self.inner.borrow().region.start().into_raw_value() % PAGE_SIZE == 0
        }
    }
}
```

---

### Phase 3: Virtual Memory Management Verification (3-4 weeks)

**Goal**: Verify Vmem and PageTable, prove P4-P7, P10

#### 3.1 Vmem View and Invariant

```rust
verus! {
    pub struct VmemView {
        pub user_mappings: Map<int, int>,    // VirtualAddr -> FrameAddr
        pub kernel_mappings: Map<int, int>,
    }

    impl Vmem {
        pub closed spec fn view(&self) -> VmemView;

        pub open spec fn inv(&self) -> bool {
            // 1. User mapping addresses are in user space
            &&& forall|vaddr| self@.user_mappings.contains_key(vaddr)
                ==> Self::spec_is_user_addr(vaddr)
            // 2. Kernel mapping addresses are in kernel space
            &&& forall|vaddr| self@.kernel_mappings.contains_key(vaddr)
                ==> Self::spec_is_kernel_addr(vaddr)
            // 3. P4: User/kernel space disjoint (guaranteed by address definition)
            // 4. P5: User mappings don't point to kernel frames
            &&& forall|vaddr| self@.user_mappings.contains_key(vaddr)
                ==> !is_kernel_frame(self@.user_mappings[vaddr])
            // 5. Page table structure is valid
            &&& self.pgdir.inv()
        }
    }
}
```

#### 3.2 map/unmap Verification

```rust
verus! {
    impl Vmem {
        pub fn map(&mut self, vaddr: VirtualAddress, frame: FrameAddress, ...)
            -> (result: Result<(), Error>)
            requires
                old(self).inv(),
                Self::spec_is_user_addr(vaddr.into_raw_value() as int),
                !is_kernel_frame(frame.into_raw_value() as int),  // P5 precondition
            ensures
                self.inv(),
                result.is_ok() ==> {
                    // P6: Lookup succeeds after mapping
                    self.spec_lookup(vaddr) == Some(frame)
                    // Frame condition: other mappings unchanged
                    && forall|v| v != vaddr
                        ==> self.spec_lookup(v) == old(self).spec_lookup(v)
                }
        ;
    }
}
```

---

### Phase 4: Address Type Verification (1-2 weeks)

**Goal**: Verify address type conversion correctness

#### 4.1 Address Type Specifications

```rust
verus! {
    impl PageAligned<PhysicalAddress> {
        pub open spec fn inv(&self) -> bool {
            self.addr.into_raw_value() % PAGE_SIZE == 0
        }
    }

    // P4 concrete implementation
    pub open spec fn spec_is_user_addr(addr: int) -> bool {
        USER_BASE <= addr && addr < USER_END
    }

    pub open spec fn spec_is_kernel_addr(addr: int) -> bool {
        KERNEL_BASE <= addr && addr < KERNEL_END
    }

    // Prove P4
    proof fn lemma_user_kernel_disjoint()
        ensures
            forall|addr: int|
                spec_is_user_addr(addr) ==> !spec_is_kernel_addr(addr)
    {
        // Follows directly from USER_END <= KERNEL_BASE
    }
}
```

---

### Phase 5: Composition Proofs and Integration (2-3 weeks)

**Goal**: Prove top-level properties P1-P10 hold

#### 5.1 Composition Proof Structure

```rust
// proofs/isolation.rs
verus! {
    /// Prove P5: User processes cannot access kernel memory
    /// This requires combining Vmem::inv() and address type specs
    proof fn theorem_user_kernel_isolation(vmem: Vmem)
        requires
            vmem.inv(),
        ensures
            forall|user_vaddr: int|
                spec_is_user_addr(user_vaddr)
                && vmem@.user_mappings.contains_key(user_vaddr)
                ==> !is_kernel_frame(vmem@.user_mappings[user_vaddr])
    {
        // Follows directly from constraints in vmem.inv()
    }
}

// proofs/allocator.rs
verus! {
    /// Prove P1: No double allocation
    /// Combine two consecutive allocations
    proof fn theorem_no_double_allocation(allocator: FrameAllocator)
        requires
            allocator.inv(),
        ensures
            forall|frame: int|
                allocator@.is_allocated(frame)
                ==> allocator.alloc() != Ok(frame_from_idx(frame))
    {
        // Follows from alloc()'s ensures "!old(self)@.is_allocated(idx)"
    }
}
```

#### 5.2 Establish Complete Trust Chain

```
Top-Level Theorems (Phase 5)
├── theorem_user_kernel_isolation ─────── proves P5
│   └── depends on Vmem::inv() ─────────── Phase 3
│       └── depends on PageTable::inv()
│
├── theorem_no_double_allocation ──────── proves P1
│   └── depends on FrameAllocator::alloc() ─── Phase 1
│       └── depends on Bitmap::inv() ────────── Already verified
│
└── theorem_allocation_failure_safe ───── proves P9
    └── depends on all allocator ensures
```

#### 5.3 Continuous Integration

```yaml
# .github/workflows/verus-verify.yml
name: Verus Verification
on: [push, pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Verus
        run: cargo install verus
      - name: Run verification
        run: |
          verus --crate-type lib src/kernel/src/mm/specs/mod.rs
          verus --crate-type lib verus/slab/lib.rs
          verus --crate-type lib verus/kheap/lib.rs
```

---

## 5. Refinement Verification Explained

### 5.1 What is Refinement?

Refinement proves that the implementation is a "refinement" of the specification—the implementation's behavior is a subset of the specification's allowed behaviors.

```
Abstract Spec                        Concrete Implementation
┌─────────────────┐                 ┌─────────────────┐
│ abstract_alloc  │                 │ FrameAllocator  │
│ (pure function, │◄── refines ────│ ::alloc()       │
│  no side effects)│                │ (has side effects)│
└─────────────────┘                 └─────────────────┘
```

### 5.2 How to Express Refinement in Verus

Refinement is implicitly expressed through three elements:

| Element | Purpose | Example |
|---------|---------|---------|
| `view()` | Abstraction function, maps concrete to abstract state | `fn view(&self) -> View` |
| `inv()` | Coupling invariant, connects abstract and concrete | `spec fn inv(&self) -> bool` |
| `ensures` | Constrains View changes to conform to abstract operation | `ensures self@ == abstract_op(old(self)@)` |

### 5.3 Complete Refinement Proof Pattern

```rust
verus! {
    impl FrameAllocator {
        // 1. Define abstract state
        pub closed spec fn view(&self) -> FrameAllocatorView;

        // 2. Define coupling invariant
        pub open spec fn inv(&self) -> bool {
            // Abstract state consistent with concrete state
            forall|i| self@.is_allocated(i) <==> self.bitmap.is_bit_set(i)
        }

        // 3. Function specification proves Refinement
        pub fn alloc(&mut self) -> Result<FrameAddress, Error>
            requires old(self).inv(),
            ensures
                // Invariant preserved
                self.inv(),
                // View change conforms to abstract operation
                (self@, result.ok()) == abstract_alloc(old(self)@)
        ;
    }
}
```

### 5.4 Explicit vs Implicit Refinement

| Approach | Description | Use Case |
|----------|-------------|----------|
| **Implicit** (Nanvix current) | Constrain View changes in `ensures` | Most cases |
| **Explicit** | Define `abstract_op` and prove equivalence | Need stronger guarantees |

Explicit approach example:

```rust
verus! {
    // Explicitly prove implementation satisfies abstract specification
    proof fn alloc_refines_abstract(allocator: &mut FrameAllocator)
        requires old(allocator).inv()
        ensures
            allocator.inv(),
            // Explicitly state: implementation behavior equals abstract operation
            (allocator@, allocator.alloc().ok().map(|f| f.into_idx()))
                == abstract_alloc(old(allocator)@)
    ;
}
```

---

## 6. Best Practices and Guidelines

### 6.1 Reusable Patterns

| Pattern | Description | Application in Nanvix |
|---------|-------------|----------------------|
| Ownership Tracking | Track resource ownership with ghost state | Create VmemOwners for Vmem |
| Type Representation | Define representation traits for type conversions | Use for address type conversions |
| Invariant Trait | Standardize invariant definition | All core data structures implement `inv()` |
| Ghost State (`Tracked<T>`) | Track allocator state at spec level | Track allocation sets |

### 6.2 Issues to Avoid

| Issue | Description | Recommendation |
|-------|-------------|----------------|
| Too many `assume` | Breaks verification completeness | Target: 0 assume |
| Too many `external_body` | Expands TCB | Verify function bodies where possible |
| Unclear TCB | Don't know what's trusted | Document all assumptions |
| Missing `inv()` preservation | Incomplete Refinement | Every method verifies `inv()` preservation |

### 6.3 Verification Quality Standards

Each module's verification should satisfy:

```
✅ 0 assume statements
✅ 0 admit statements
✅ external_body only for true external dependencies
✅ All public functions have complete requires/ensures
✅ inv() preserved after all operations
✅ VERIFICATION_SUMMARY.md records verification status
```

---

## 7. Verification Priority and Timeline

### 7.1 Priority Ordering

| Priority | Module | Properties | Est. Time | Dependencies |
|----------|--------|------------|-----------|--------------|
| **P0** | FrameAllocator | P1-P3, P8-P9 | 2 weeks | Bitmap ✅ |
| **P0** | Upool / Kpool | P1-P3, P8-P9 | 1 week | FrameAllocator, Slab ✅ |
| **P1** | Vmem::map/unmap | P6-P7 | 2 weeks | PageTable |
| **P1** | PageTable | P6-P7 | 2 weeks | - |
| **P2** | Vmem::is_user_addr | P4-P5 | 1 week | Address types |
| **P2** | copy_from/to_user | P10 | 1 week | Vmem |
| **P3** | Address type conversion | - | 1 week | - |
| **P4** | Composition proofs | P1-P10 | 2 weeks | All modules |

### 7.2 Milestones

```
Week 1-2:   Phase 0 complete (Specification design)
Week 3-5:   Phase 1 complete (FrameAllocator verification)
Week 6-7:   Phase 2 complete (Memory pool verification)
Week 8-11:  Phase 3 complete (Vmem verification)
Week 12-13: Phase 4 complete (Address types)
Week 14-16: Phase 5 complete (Composition proofs)
```

---

## 8. Expected Outcomes

Upon completion, Nanvix will have:

### 8.1 Formal Safety Guarantees

- ✅ Memory isolation proof (P4, P5)
- ✅ Allocator correctness proof (P1-P3, P8-P9)
- ✅ Page table consistency proof (P6-P7)
- ✅ Copy safety proof (P10)

### 8.2 Verifiable Specification Documentation

- Pre/post conditions for each function
- System-level invariants
- View types as formal documentation

### 8.3 Continuous Verification Pipeline

- CI integrated Verus checking
- Automatic re-verification on code changes
- Verification failures block merges

### 8.4 Verification Metrics Targets

| Metric | Target |
|--------|--------|
| Verification coverage | 100% of memory management core functions |
| assume count | 0 |
| admit count | 0 |
| external_body | Only for true external dependencies |
| Top-level property proofs | All P1-P10 complete |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **View** | Abstract state type that abstracts away implementation details |
| **Invariant (inv)** | Predicate connecting abstract and concrete states |
| **Refinement** | Implementation is a refinement of the specification |
| **Coupling Invariant** | Predicate describing relationship between abstract and concrete states |
| **Frame Condition** | Operation only affects specified parts, others unchanged |
| **TCB** | Trusted Computing Base, code that must be trusted |
| **external_body** | Verus annotation marking unverified function body |
| **assume** | Verus annotation asserting something as true without proof |

## Appendix B: References

- [Verus Official Documentation](https://verus-lang.github.io/verus/)
- Nanvix Existing Verification: `verus/slab/`, `verus/kheap/`, `verus/bitmap/`
