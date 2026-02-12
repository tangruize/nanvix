# Bug Report: Slab::deallocate() Missing Alignment Check — Silent Wrong-Block Deallocation

> **Severity:** MEDIUM-HIGH
> **Status:** Confirmed real bug, not yet fixed in production code; fix exists in Verus verification model
> **Discovery Method:** Structural — Verus formal verification's `is_valid_addr` spec requires alignment,
> forcing the AI prover to add a runtime alignment check that the original code lacks
> **Affected File:** [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) (lines 203–215)
> **Verified File:** [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) (lines 792–795, fix present)
> **Related to:** [#1351](https://github.com/nanvix/nanvix/issues/1351) (from_raw_parts underflow — same module, same verification effort)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Trigger Scenario and Impact](#3-trigger-scenario-and-impact)
4. [How Verus Verification Discovered This Bug](#4-how-verus-verification-discovered-this-bug)
5. [The Verified Fix](#5-the-verified-fix)
6. [Proposed Fix for Production Code](#6-proposed-fix-for-production-code)
7. [File Reference Index](#7-file-reference-index)

---

## 1. Executive Summary

The `Slab::deallocate()` function accepts any pointer within the data region's address range
but does **not check** whether the pointer is aligned to `block_size`. When an unaligned pointer
is passed, the integer division `(ptr - data_addr) / block_size` silently truncates, computing
the wrong block index. The function then frees a block that the caller did not intend to free,
which can lead to **double-allocation** (two callers receiving the same block) or **freeing an
unallocated block** (corrupting the bitmap).

This bug was discovered during Verus formal verification. The verified slab's `deallocate`
precondition requires `is_valid_addr(addr)`, which includes an alignment check:
`(addr - data_addr) % block_size == 0`. To provide defense-in-depth for unverified callers,
the AI prover also added a runtime alignment guard. The original production code has no such
check.

---

## 2. The Bug

### Root Cause

In [`src/libs/slab/src/lib.rs` (lines 203–215)](../src/libs/slab/src/lib.rs#L203-L215), the
`deallocate()` function performs bounds checking but **no alignment checking**:

```rust
// lib.rs:203-215
pub unsafe fn deallocate(&mut self, ptr: *const u8) -> Result<(), Error> {
    // Check if the pointer lies in a memory region that is not managed by this allocator.
    if ptr < self.data_addr
        || ptr >= unsafe { self.data_addr.add(self.num_data_blocks * self.block_size) }
    {
        return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
    }

    // Compute the block index.
    let index: usize = self.num_index_blocks
        + unsafe { ptr.offset_from_unsigned(self.data_addr) } / self.block_size;
    //                                                         ^^^^^^^^^^^^^^^^
    //     Integer division truncates — unaligned ptr maps to wrong block!

    // ...frees block at `index`...
}
```

### The Arithmetic Problem

Given `data_addr = 0x1000`, `block_size = 4096`:

| Input `ptr` | `ptr - data_addr` | `/ block_size` | Actual block freed | Intended? |
|-------------|--------------------|-----------------|--------------------|-----------|
| `0x1000` | `0` | `0` | Block 0 | ✅ Correct |
| `0x2000` | `4096` | `1` | Block 1 | ✅ Correct |
| `0x1001` | `1` | `0` | **Block 0** | ❌ Wrong! |
| `0x1FFF` | `4095` | `0` | **Block 0** | ❌ Wrong! |
| `0x2001` | `4097` | `1` | **Block 1** | ❌ Wrong! |

Any address within a block's 4096-byte range maps to the same block index. If the caller
passes `ptr = 0x1001` (perhaps due to pointer arithmetic on a sub-object within a block), the
function silently frees Block 0 — which may be the wrong block, or may already be free.

### Consequences

1. **Double allocation:** If Block 0 is freed by mistake, the allocator can hand it out again
   to a new caller while the original holder still holds a pointer to it. Two callers now
   share the same memory — a use-after-free/aliasing violation.

2. **Bitmap corruption:** If the truncated index points to an already-free block, `self.index.test(index)`
   returns false, and the function returns `Err`. But this masks the real error — the caller
   thinks their deallocation failed when in fact the address was simply misaligned.

3. **Silent data corruption:** The most dangerous case: the truncated block index happens to
   point to a different, legitimately allocated block. The function frees it silently, and the
   original block remains allocated but the caller believes it was freed. A later allocation
   reuses the freed block, leading to overlapping memory regions.

---

## 3. Trigger Scenario and Impact

### How It Can Be Triggered

```rust
let mut slab = unsafe { Slab::from_raw_parts(addr, len, 4096) }.unwrap();

// Allocate a block — returns pointer to start of block.
let block_ptr: *mut u8 = slab.allocate().unwrap();
// block_ptr is, say, 0x2000 (start of Block 1)

// Caller computes a sub-pointer (e.g., to access a field within the block).
let field_ptr: *const u8 = unsafe { block_ptr.add(16) };
// field_ptr = 0x2010

// Mistakenly passes field_ptr to deallocate instead of block_ptr.
unsafe { slab.deallocate(field_ptr) }.unwrap();
// ↑ Silently frees Block 1 (correct block by luck, since 0x2010 / 4096 = 1).
// But with different offsets, it could free the WRONG block.
```

A more dangerous scenario with `block_size = 8`:

```rust
let block0: *mut u8 = slab.allocate().unwrap();  // 0x1000
let block1: *mut u8 = slab.allocate().unwrap();  // 0x1008

// Caller adds 1 to block1's pointer (off-by-one error).
let bad_ptr: *const u8 = unsafe { block1.add(1) };  // 0x1009

unsafe { slab.deallocate(bad_ptr) }.unwrap();
// (0x1009 - 0x1000) / 8 = 1 → frees Block 1 (happens to be correct).
// But if bad_ptr were 0x100F: (0x100F - 0x1000) / 8 = 1 → still Block 1.
// If bad_ptr were 0x1010: (0x1010 - 0x1000) / 8 = 2 → frees Block 2!
```

### Current Risk Assessment

| Aspect | Rating |
|--------|--------|
| Severity | MEDIUM-HIGH |
| Exploitability | Requires passing a misaligned pointer (common in C FFI or pointer arithmetic) |
| Current risk | LOW-MEDIUM (kernel callers use `allocate`'s return value directly) |
| Future risk | HIGH (any C interop, sub-object pointers, or off-by-one errors trigger it) |
| Consequences | Double-allocation, memory aliasing, silent data corruption |

---

## 4. How Verus Verification Discovered This Bug

### The `is_valid_addr` Specification

The verified slab defines a `is_valid_addr` spec function that captures what a "valid slab
address" means ([`verus/split/libs/slab/lib.spec.rs`](../verus/split/libs/slab/lib.spec.rs),
lines 56–60):

```rust
pub open spec fn is_valid_addr(&self, addr: int) -> bool {
    &&& addr >= self.data_addr
    &&& addr < self.data_addr + self.num_data_blocks * self.block_size
    &&& (addr - self.data_addr) % self.block_size == 0  // ← Alignment required!
}
```

The `deallocate` function's precondition requires `is_valid_addr(addr)`:

```rust
pub fn deallocate(&mut self, addr: usize) -> (result: Result<(), Error>)
    requires
        old(self).inv(),
        old(self)@.is_valid_addr(addr as int),
```

### Why the Runtime Check Was Added

Even though the precondition guarantees alignment for verified callers, the AI prover added
a runtime guard for **defensive programming** — protecting against unverified callers:

```rust
// Issue 3 FIX: Keep runtime bounds check for defensive programming.
// This protects against unverified callers that may violate preconditions.
```

Without this check, the `(addr - self.data_addr) / self.block_size` computation could produce
a wrong index, and the proof would not hold for unverified callers. The AI prover recognized
that the original code's safety depends entirely on callers passing correctly aligned
pointers, with no enforcement mechanism.

### The Reviewer Confirmed the Risk

The Opus reviewer ([`histories/reviewers/5-slab-opus.md`](../histories/reviewers/5-slab-opus.md))
explicitly noted the underflow risk in `deallocate`:

> "In `deallocate`, the calculation `let index = self.num_index_blocks + (addr - self.data_addr)
> / self.block_size;` is safe *because* of the precondition `addr >= self.data_addr`. In the
> absence of verification (or if the precondition were removed), `addr - self.data_addr` could
> underflow."

The alignment issue is a closely related problem — even when `addr >= self.data_addr`, if
`addr` is not aligned, the division truncates to the wrong index.

---

## 5. The Verified Fix

In [`verus/split/libs/slab/lib.rs` (lines 772–795)](../verus/split/libs/slab/lib.rs#L772-L795):

```rust
pub fn deallocate(&mut self, addr: usize) -> (result: Result<(), Error>)
{
    // Check if the address is below the data region.
    if addr < self.data_addr {
        return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds (below data region)"));
    }

    // Check if the address is beyond the data region.
    let data_region_size: usize = self.num_data_blocks * self.block_size;
    let data_region_end: usize = self.data_addr + data_region_size;
    if addr >= data_region_end {
        return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds (beyond data region)"));
    }

    // ← THE FIX: Check if the address is properly aligned to block size.
    if (addr - self.data_addr) % self.block_size != 0 {
        return Err(Error::new(ErrorCode::BadAddress, "unaligned block address"));
    }

    let index: usize = self.num_index_blocks + (addr - self.data_addr) / self.block_size;
    // ...
}
```

The verified version splits the bounds check into three distinct parts:
1. **Lower bound:** `addr >= data_addr`
2. **Upper bound:** `addr < data_addr + num_data_blocks * block_size`
3. **Alignment:** `(addr - data_addr) % block_size == 0` ← **New!**

---

## 6. Proposed Fix for Production Code

Add a single alignment check after the existing bounds check in
[`src/libs/slab/src/lib.rs` (after line 210)](../src/libs/slab/src/lib.rs#L210):

```rust
pub unsafe fn deallocate(&mut self, ptr: *const u8) -> Result<(), Error> {
    // Check if the pointer lies in a memory region that is not managed by this allocator.
    if ptr < self.data_addr
        || ptr >= unsafe { self.data_addr.add(self.num_data_blocks * self.block_size) }
    {
        return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
    }

    // Check if the pointer is aligned to block size.
    if unsafe { ptr.offset_from_unsigned(self.data_addr) } % self.block_size != 0 {
        return Err(Error::new(ErrorCode::BadAddress, "unaligned block address"));
    }

    // Compute the block index.
    let index: usize = self.num_index_blocks
        + unsafe { ptr.offset_from_unsigned(self.data_addr) } / self.block_size;

    // ... rest unchanged ...
}
```

This is a one-line behavioral change that prevents silent wrong-block deallocation.

---

## 7. File Reference Index

| File | Role | Key Lines |
|------|------|-----------|
| [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) | **Production code (buggy)** | L203–215: no alignment check in deallocate |
| [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) | **Verified code (fixed)** | L792–795: alignment check added |
| [`verus/split/libs/slab/lib.spec.rs`](../verus/split/libs/slab/lib.spec.rs) | Specification | L56–60: `is_valid_addr` requires alignment |
| [`histories/reviewers/5-slab-opus.md`](../histories/reviewers/5-slab-opus.md) | Opus review | Notes underflow risk in deallocate arithmetic |
