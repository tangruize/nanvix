# Bug Report: Slab::deallocate() Bounds Check Self-Overflow — `num_data_blocks * block_size` Can Wrap

> **Severity:** LOW-MEDIUM
> **Status:** Confirmed potential issue, not yet fixed in production code; verified code proves
> the overflow cannot happen via invariant
> **Discovery Method:** Structural — Verus formal verification requires explicit proof that the
> bounds-check arithmetic does not overflow
> **Affected File:** [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) (line 207)
> **Verified File:** [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) (lines 783–787, invariant-based proof)
> **Related to:** [#1351](https://github.com/nanvix/nanvix/issues/1351) (from_raw_parts underflow — if triggered, makes this bug exploitable)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Trigger Scenario and Impact](#3-trigger-scenario-and-impact)
4. [How Verus Verification Addressed This](#4-how-verus-verification-addressed-this)
5. [Why This Is Lower Severity](#5-why-this-is-lower-severity)
6. [Proposed Fix for Production Code](#6-proposed-fix-for-production-code)
7. [File Reference Index](#7-file-reference-index)

---

## 1. Executive Summary

The `Slab::deallocate()` function computes the upper bound of the data region as
`self.data_addr.add(self.num_data_blocks * self.block_size)` for bounds checking. If
`num_data_blocks * block_size` overflows `usize`, the computed upper bound wraps to a small
value, effectively **disabling the upper-bound safety check**. Any pointer above `data_addr`
would then pass the bounds check, allowing out-of-bounds bitmap operations.

In a correctly constructed slab, this overflow cannot happen because the data region must fit
within the original memory allocation. However, if the slab's internal state is corrupted
(e.g., by the underflow bug #1351 producing a huge `num_data_blocks`), this bounds check
silently becomes a no-op.

---

## 2. The Bug

### Root Cause

In [`src/libs/slab/src/lib.rs` (line 207)](../src/libs/slab/src/lib.rs#L207):

```rust
// lib.rs:206-210
if ptr < self.data_addr
    || ptr >= unsafe { self.data_addr.add(self.num_data_blocks * self.block_size) }
    //                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                                    If num_data_blocks is corrupted (e.g., ~usize::MAX
    //                                    from the underflow bug), this multiplication wraps,
    //                                    and the bounds check becomes useless.
{
    return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
}
```

### The Overflow Chain

This is a **cascading failure** from the `from_raw_parts` underflow bug (#1351):

```
Bug #1351 triggers:
  num_data_blocks = total_num_blocks - num_index_blocks   ← underflows to ~usize::MAX

This bug cascades:
  num_data_blocks * block_size = ~usize::MAX * block_size ← wraps to small value
  data_addr.add(small_value)                              ← upper bound is too low

Result:
  The bounds check rejects VALID pointers (false negative for high addresses)
  OR accepts INVALID pointers (false positive if wrap produces value above data_addr)
```

### Specific Overflow Example

If `num_data_blocks = usize::MAX` (from underflow) and `block_size = 8`:

```
num_data_blocks * block_size = usize::MAX * 8
                             = 0xFFFF_FFFF_FFFF_FFF8 (on 64-bit)
                             wraps to: 0xFFFF_FFFF_FFFF_FFF8 (actually fits in this case)
```

But with `block_size = 16`:
```
usize::MAX * 16 = 0xFFFF_FFFF_FFFF_FFF0 (wraps)
data_addr + 0xFFFF_FFFF_FFFF_FFF0 → wraps to data_addr - 16
```

The bounds check `ptr >= data_addr - 16` would accept almost **all** pointers below
`data_addr`, which should be rejected.

---

## 3. Trigger Scenario and Impact

### Triggering Conditions

This bug is only exploitable when `num_data_blocks` has been corrupted by another bug
(primarily #1351). In a correctly initialized slab, the invariant
`data_addr + num_data_blocks * block_size <= addr + len` holds, and the multiplication
cannot overflow because `addr + len` fits in `usize`.

### Impact Assessment

| Aspect | Rating |
|--------|--------|
| Severity | LOW-MEDIUM |
| Standalone risk | NONE (cannot trigger without prior corruption) |
| Cascading risk | HIGH (amplifies the damage from #1351) |
| Defense-in-depth value | MEDIUM (fixing this limits blast radius of other bugs) |

---

## 4. How Verus Verification Addressed This

The verified code takes a **fundamentally different approach**. Instead of hoping the
arithmetic is safe, it **proves** it via the slab invariant:

```rust
// verus/split/libs/slab/lib.rs:783-787
proof {
    assert((self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int)
        <= usize::MAX as int);
}
let data_region_size: usize = self.num_data_blocks * self.block_size;
let data_region_end: usize = self.data_addr + data_region_size;
```

The slab invariant ([`verus/split/libs/slab/lib.spec.rs`](../verus/split/libs/slab/lib.spec.rs),
lines 194–196) explicitly guarantees:

```rust
// Invariant conditions:
&&& (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
&&& (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int)
    <= usize::MAX as int
```

This invariant is **established** in `from_raw_parts` (with the overflow checks from the
Multiplication Overflow fix) and **preserved** by `allocate` and `deallocate` (which only
modify the bitmap, not the size fields).

The key insight is that the verified version **eliminates the possibility** of overflow at the
source (construction time) rather than checking for it at every use site.

---

## 5. Why This Is Lower Severity

1. **Not independently triggerable:** A correctly constructed slab has
   `num_data_blocks * block_size ≤ len < i32::MAX`, so the multiplication fits in `usize`.

2. **Defense-in-depth only:** Fixing this provides protection against cascading failures from
   other bugs, but does not fix a standalone vulnerability.

3. **The real fix is upstream:** Fixing #1351 (the underflow bug) prevents `num_data_blocks`
   from being corrupted, which eliminates this overflow scenario entirely.

However, following the defense-in-depth principle, the bounds check should still be safe
against corrupted internal state.

---

## 6. Proposed Fix for Production Code

### Option A: Use Checked Arithmetic (Recommended)

```rust
pub unsafe fn deallocate(&mut self, ptr: *const u8) -> Result<(), Error> {
    // Compute upper bound with overflow protection.
    let data_region_size: usize = self.num_data_blocks.checked_mul(self.block_size)
        .ok_or(Error::new(ErrorCode::BadAddress, "internal overflow"))?;
    let data_region_end: *mut u8 = self.data_addr.wrapping_add(data_region_size);

    if ptr < self.data_addr || ptr >= data_region_end || data_region_end < self.data_addr {
        return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
    }

    // ... rest unchanged ...
}
```

### Option B: Establish the Invariant at Construction Time

Fix #1351 (the underflow bug) and add a debug assertion in `from_raw_parts`:

```rust
debug_assert!(
    (data_addr as usize).checked_add(num_data_blocks * block_size).is_some(),
    "data region exceeds address space"
);
```

This ensures that if the construction is correct, `deallocate`'s arithmetic is always safe.

---

## 7. File Reference Index

| File | Role | Key Lines |
|------|------|-----------|
| [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) | **Production code (vulnerable)** | L207: unchecked `num_data_blocks * block_size` in bounds check |
| [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) | **Verified code (safe)** | L783–787: invariant proves no overflow |
| [`verus/split/libs/slab/lib.spec.rs`](../verus/split/libs/slab/lib.spec.rs) | Specification | L194–196: overflow-free invariant conditions |
| [`SlabUnderflowBugReport.md`](../SlabUnderflowBugReport.md) | Related bug | #1351 — corrupted `num_data_blocks` enables this overflow |
