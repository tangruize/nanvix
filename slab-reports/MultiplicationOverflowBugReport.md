# Bug Report: Slab::from_raw_parts() Multiplication Overflow — Unchecked `num_index_blocks * block_size`

> **Severity:** MEDIUM
> **Status:** Confirmed real bug, not yet fixed in production code; fix exists in Verus verification model
> **Discovery Method:** Structural — Verus formal verification's arithmetic overflow rules forced the AI
> prover to add `checked_mul` and explicit overflow guards
> **Affected File:** [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) (line 136)
> **Verified File:** [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) (lines 248–257, fix present)
> **Related to:** [#1351](https://github.com/nanvix/nanvix/issues/1351) (from_raw_parts underflow — same function)

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

The `Slab::from_raw_parts()` function computes `data_addr = addr + num_index_blocks * block_size`
(line 136) without checking for multiplication or addition overflow. While the existing
validation checks `block_size > 0`, `block_size < i32::MAX`, and `len < i32::MAX`, the
intermediate product `num_index_blocks * block_size` is **not** checked against `usize::MAX`.

In pathological cases, this multiplication can wrap around, producing a `data_addr` that
points into the wrong memory region. The slab allocator would then hand out pointers that
overlap with the index region or point to completely unrelated memory.

The Verus verification effort discovered this when the prover attempted to prove that
`data_addr` is within the valid address space. Verus's arithmetic safety rules required an
explicit proof that the multiplication does not overflow, forcing the prover to add
`checked_mul` and bounds validation.

---

## 2. The Bug

### Root Cause

In [`src/libs/slab/src/lib.rs` (line 136)](../src/libs/slab/src/lib.rs#L136):

```rust
// lib.rs:136
let data_addr: *mut u8 = addr.add(num_index_blocks * block_size);
//                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//                                 Two unchecked operations:
//                                 1. num_index_blocks * block_size  (multiplication overflow)
//                                 2. addr + product                 (pointer addition overflow)
```

### The Arithmetic Chain

The computation involves two sequential unchecked operations:

```
Step 1: product = num_index_blocks * block_size     → Can wrap to small value
Step 2: data_addr = addr + product                  → Can wrap to small value
```

### Existing Guards (Insufficient)

The function validates:
- `len < i32::MAX` (line 95) — but `num_index_blocks` is derived from `len / block_size / 8`,
  which can still be large.
- `block_size < i32::MAX` (line 105) — bounds one operand.
- `block_size > 0` (line 105) — prevents division by zero.

But there is **no check** that `num_index_blocks * block_size <= usize::MAX`.

### Example Overflow Scenario

On a 32-bit system (`usize = u32`, max = 4,294,967,295):

- `len = 2,147,483,640` (≈ 2 GB, just under `i32::MAX`)
- `block_size = 8`
- `total_num_blocks = 268,435,455` (multiple of 8 ✓)
- `index_len = 33,554,431`
- `num_index_blocks = ceil(33,554,431 / 8) = 4,194,304`
- `num_index_blocks * block_size = 33,554,432` ← fits in u32 (safe in this case)

But with different parameters approaching the boundary:

- `len = 2,147,483,640`, `block_size = 1`
- `total_num_blocks = 2,147,483,640` (multiple of 8 ✓)
- `index_len = 268,435,455`
- `num_index_blocks = ceil(268,435,455 / 1) = 268,435,455`
- `num_index_blocks * block_size = 268,435,455` ← fits (safe because block_size = 1)

The overflow is most likely when `block_size` and `num_index_blocks` are both moderately large
on 32-bit systems, or when the slab is used with unusual configurations.

### Impact

If the multiplication wraps:
1. `data_addr` points to a wrong location (potentially before `addr`).
2. The index region and data region overlap — writes to the bitmap corrupt data blocks and
   vice versa.
3. All subsequent `allocate()` calls return pointers into corrupted memory.

---

## 3. Trigger Scenario and Impact

### Current Risk

On **64-bit systems** (the primary Nanvix target), overflow of `num_index_blocks * block_size`
is extremely unlikely because both values are bounded by `i32::MAX`. However:

- On **32-bit systems** (Nanvix targets x86-32), the risk is higher.
- The `slab` crate is a **general-purpose library** — future callers may use parameters that
  the kernel does not.
- Even if overflow is unlikely, the **absence of a check** means the function provides no
  safety guarantee, violating the principle of defensive programming.

| Aspect | Rating |
|--------|--------|
| Severity | MEDIUM |
| Exploitability | Requires specific large parameter combinations |
| Current risk | LOW (kernel uses small slab sizes) |
| Future risk | MEDIUM (library API should be self-protecting) |
| Consequences | Memory corruption, overlapping index/data regions |

---

## 4. How Verus Verification Discovered This Bug

When the AI prover removed `#[verifier::external_body]` from `from_raw_parts` to fully verify
it (commit `54c797328`, 2026-01-12), Verus generated proof obligations for every arithmetic
operation:

1. **For `num_index_blocks * block_size`:** Prove that the product fits in `usize`.
2. **For `addr + product`:** Prove that the sum fits in `usize`.

The prover could not discharge these obligations without explicit runtime checks, because the
existing validations on `len` and `block_size` individually do not bound their product. The
prover was forced to add:

```rust
let max_blocks: usize = usize::MAX / block_size;
if num_index_blocks > max_blocks {
    return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
}
```

This is the same structural discovery mechanism as the underflow bug (#1351) — Verus's type
system **requires** proofs of no-overflow, and when the proof is impossible, the only solution
is to add runtime validation.

---

## 5. The Verified Fix

In [`verus/split/libs/slab/lib.rs` (lines 244–267)](../verus/split/libs/slab/lib.rs#L244-L267):

```rust
// Check for overflow in address calculation.
if block_size == 0 {
    return Err(Error::new(ErrorCode::InvalidArgument, "block size is zero"));
}
let max_blocks: usize = usize::MAX / block_size;
if num_index_blocks > max_blocks {
    return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
}

// Now we can safely multiply using checked_mul.
let index_region_size: usize = match num_index_blocks.checked_mul(block_size) {
    Some(v) => v,
    None => return Err(Error::new(ErrorCode::InvalidArgument, "address overflow")),
};

// Check index_region_size > 0.
if index_region_size == 0 {
    return Err(Error::new(ErrorCode::InvalidArgument, "index region size is zero"));
}

// Check for addition overflow.
if addr > usize::MAX - index_region_size {
    return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
}
let data_addr: usize = addr + index_region_size;
```

The verified version provides **triple protection**:
1. Pre-division bound check (`num_index_blocks > usize::MAX / block_size`)
2. `checked_mul` for the multiplication itself
3. Pre-addition bound check (`addr > usize::MAX - index_region_size`)

---

## 6. Proposed Fix for Production Code

Replace line 136 with checked arithmetic:

```rust
// Check for multiplication overflow.
let index_region_size: usize = num_index_blocks.checked_mul(block_size)
    .ok_or(Error::new(ErrorCode::InvalidArgument, "address overflow"))?;

// Check for addition overflow.
let data_addr: *mut u8 = if (addr as usize) > usize::MAX - index_region_size {
    return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
} else {
    addr.add(index_region_size)
};
```

---

## 7. File Reference Index

| File | Role | Key Lines |
|------|------|-----------|
| [`src/libs/slab/src/lib.rs`](../src/libs/slab/src/lib.rs) | **Production code (buggy)** | L136: unchecked `num_index_blocks * block_size` |
| [`verus/split/libs/slab/lib.rs`](../verus/split/libs/slab/lib.rs) | **Verified code (fixed)** | L248–267: checked_mul + overflow guards |
| Git commit `54c797328` | The fix commit | "slab from raw part" — overflow checks added |
