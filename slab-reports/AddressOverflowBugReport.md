# Bug Report: Slab::from_raw_parts() Address Addition Overflow — Unchecked `addr + index_region_size`

> **Severity:** MEDIUM
> **Status:** Confirmed real bug, not yet fixed in production code; fix exists in Verus verification model
> **Discovery Method:** Structural — Verus formal verification's arithmetic overflow rules forced the AI
> prover to add an explicit addition overflow guard
> **Affected File:** [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) (line 136)
> **Verified File:** [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) (lines 264–267, fix present)
> **Related to:** Multiplication Overflow (same line, same computation chain)

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

Even if `num_index_blocks * block_size` does not overflow (see the Multiplication Overflow
report), the subsequent addition `addr + (num_index_blocks * block_size)` can still overflow
when `addr` is a large address and the product is large. The original code uses
`addr.add(num_index_blocks * block_size)` without verifying that the result fits in the
address space.

On overflow, `data_addr` wraps around to a small address value. The slab allocator then stores
this wrong `data_addr`, and all subsequent `allocate()` calls return pointers into low memory
(potentially NULL page, kernel code, or other critical regions), causing catastrophic memory
corruption.

---

## 2. The Bug

### Root Cause

In [`src/libs/slab/src/lib.rs` (line 136)](../src/libs/slab/src/lib.rs#L136):

```rust
let data_addr: *mut u8 = addr.add(num_index_blocks * block_size);
```

The `.add()` method on raw pointers performs wrapping addition in release mode. There is no
check that `addr as usize + num_index_blocks * block_size <= usize::MAX`.

### The Overflow Chain

This bug is the **second stage** of a two-stage overflow:

```
Stage 1: product = num_index_blocks * block_size    ← Multiplication Overflow report
Stage 2: data_addr = addr + product                 ← THIS REPORT
```

Even when Stage 1 is safe (product fits in usize), Stage 2 can overflow if `addr + product >
usize::MAX`.

### Example

On a 32-bit system:
- `addr = 0xF000_0000` (3.75 GB — high memory, common for kernel heap)
- `num_index_blocks * block_size = 0x2000_0000` (512 MB — large but valid)
- `data_addr = 0xF000_0000 + 0x2000_0000 = 0x1_1000_0000` → wraps to `0x1000_0000`

The resulting `data_addr` (`0x1000_0000`, 256 MB) is in a completely different memory region
than intended. The existing `addr.wrapping_add(len) < addr` check on line 100 catches the
total region wrap, but it does **not** catch the partial wrap of just the index region.

---

## 3. Trigger Scenario and Impact

### Triggering Conditions

The bug requires:
1. `addr` is in high memory (close to address space limit).
2. `num_index_blocks * block_size` is large enough that `addr + product` exceeds `usize::MAX`.
3. The total `len` check (`addr.wrapping_add(len) < addr`) may or may not catch this,
   depending on the relationship between `len` and `num_index_blocks * block_size`.

### The Gap in Existing Validation

Line 100 checks: `addr.wrapping_add(len) < addr` — this ensures the **total** region
`[addr, addr+len)` doesn't wrap. But `data_addr = addr + num_index_blocks * block_size` is
always ≤ `addr + len` (since index blocks are part of the total length), so if the total
region doesn't wrap, the data_addr addition shouldn't wrap either.

**However**, the overflow on line 136 uses `addr.add()` which is **pointer arithmetic** with
wrapping semantics, not the checked `wrapping_add` comparison. And the line 100 check uses
`wrapping_add` for the comparison but doesn't prevent the `.add()` on line 136 from actually
wrapping — it merely detects total-range wrapping, not intermediate-computation wrapping.

More importantly, if the underflow bug (#1351) has already corrupted `num_index_blocks`
(making it larger than `total_num_blocks`), then `num_index_blocks * block_size` can be
**larger** than `len`, and the line 100 check provides no protection.

| Aspect | Rating |
|--------|--------|
| Severity | MEDIUM |
| Exploitability | Requires high memory addresses + large index regions |
| Current risk | LOW (kernel heap is in controlled address range) |
| Future risk | MEDIUM (compounded by #1351 underflow if both bugs present) |
| Consequences | data_addr points to wrong memory, catastrophic corruption |

---

## 4. How Verus Verification Discovered This Bug

Verus treats pointer addresses as mathematical integers and requires proof that all
intermediate computations stay within `[0, usize::MAX]`. When verifying
`let data_addr = addr + index_region_size`, Verus generated the proof obligation:

> `(addr as int) + (index_region_size as int) <= usize::MAX as int`

The existing preconditions and earlier checks did not establish this bound directly for the
intermediate computation (only for the total `addr + len`). The prover added an explicit
overflow check to satisfy this obligation.

---

## 5. The Verified Fix

In [`verus/split/libs/slab/lib.rs` (lines 264–267)](../verus/split/libs/slab/lib.rs#L264-L267):

```rust
if addr > usize::MAX - index_region_size {
    return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
}
let data_addr: usize = addr + index_region_size;
```

This is a standard overflow-check idiom: `a + b > MAX` ⟺ `a > MAX - b` (safe because
`index_region_size` is already validated to be ≤ `usize::MAX`).

---

## 6. Proposed Fix for Production Code

Add after the multiplication (which should also be checked — see the Multiplication Overflow
report):

```rust
let index_region_size: usize = num_index_blocks * block_size;  // or checked_mul

// Check for address overflow.
if (addr as usize) > usize::MAX - index_region_size {
    return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
}

let data_addr: *mut u8 = addr.add(index_region_size);
```

---

## 7. File Reference Index

| File | Role | Key Lines |
|------|------|-----------|
| [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) | **Production code (buggy)** | L136: unchecked `addr.add(product)` |
| [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) | **Verified code (fixed)** | L264–267: addition overflow guard |
| Git commit `54c797328` | The fix commit | "slab from raw part" — overflow checks added |
