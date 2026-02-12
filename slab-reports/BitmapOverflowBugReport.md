# Bug Report: Bitmap::from_raw_array() Multiplication Overflow — `array.len() * u8::BITS` Wraps Silently

> **Severity:** MEDIUM
> **Status:** Confirmed real bug in dev branch; partially mitigated in integration branch
> **Discovery Method:** Structural — Verus formal verification requires a precondition
> `array@.len() <= usize::MAX / (u8::BITS as usize)` to prevent overflow, which the dev code lacks
> **Affected File:** `src/libs/bitmap/src/lib.rs` (dev branch, line 112)
> **Verified File:** [`verus/split/libs/bitmap/lib.rs`](../verus/split/libs/bitmap/lib.rs) (line 125, precondition)
> **Also Affects:** `index()` function (dev branch, line 306) — same overflow in bounds check

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Trigger Scenario and Impact](#3-trigger-scenario-and-impact)
4. [How Verus Verification Discovered This](#4-how-verus-verification-discovered-this)
5. [The Verified Fix](#5-the-verified-fix)
6. [Current Status in Integration Branch](#6-current-status-in-integration-branch)
7. [Proposed Fix](#7-proposed-fix)
8. [File Reference Index](#8-file-reference-index)

---

## 1. Executive Summary

The `Bitmap::from_raw_array()` function computes `number_of_bits = array.len() * u8::BITS`
without checking for multiplication overflow. On systems where `array.len() > usize::MAX / 8`,
this multiplication wraps, producing a small `number_of_bits` value that is inconsistent with
the actual backing storage size. All subsequent operations (`set`, `clear`, `alloc`, `test`)
use `number_of_bits` for bounds checking, which becomes unreliable when the value is wrong.

The same overflow occurs in the `index()` function (dev branch line 306), where
`self.bits.len() * u8::BITS` is computed in the bounds check itself — if this overflows, the
bounds check silently passes for out-of-range indices, leading to out-of-bounds memory access.

The Verus verified version prevents this through a precondition:
`array@.len() <= usize::MAX / (u8::BITS as usize)`.

---

## 2. The Bug

### Root Cause — `from_raw_array` (line 112 in dev)

```rust
// dev branch: src/libs/bitmap/src/lib.rs:112
pub fn from_raw_array(mut array: RawArray<u8>) -> Self {
    Self {
        number_of_bits: array.len() * u8::BITS as usize,  // ← Can overflow!
        bits: array,
        usage: 0,
    }
}
```

If `array.len()` is very large (e.g., > `usize::MAX / 8` on 32-bit = 536,870,912), the
multiplication `array.len() * 8` wraps, and `number_of_bits` gets a small, wrong value.

### Root Cause — `index()` (line 306 in dev)

```rust
// dev branch: src/libs/bitmap/src/lib.rs:306
fn index(&self, index: usize) -> Result<(usize, usize), Error> {
    if index >= self.bits.len() * u8::BITS as usize {  // ← Can overflow!
        return Err(Error::new(ErrorCode::InvalidArgument, "index out of bounds"));
    }
    Ok(self.index_unchecked(index))
}
```

Even if `from_raw_array` is fixed, this function independently computes
`self.bits.len() * u8::BITS` on every call. If it overflows, the comparison
`index >= small_wrapped_value` can be true for legitimate indices (false positive)
or false for out-of-range indices (false negative, leading to OOB access).

The verified version avoids this by comparing against `self.number_of_bits` (a stored field)
instead of recomputing it:

```rust
// Verified: verus/split/libs/bitmap/lib.rs
if bit_index >= self.number_of_bits {  // ← Uses stored field, no multiplication
    return Err(...)
}
```

### Consequences

1. **Corrupted `number_of_bits`:** Every subsequent operation uses the wrong bitmap capacity.
   - `alloc_range` searches only within the (too small) `number_of_bits` range.
   - `set`/`clear`/`test` reject valid indices that are within the actual storage but beyond
     the wrapped `number_of_bits`.

2. **Bypassed bounds check:** If `index()` itself overflows, out-of-range bit indices pass the
   check, and `index_unchecked` computes a word/bit pair that accesses memory beyond the
   `RawArray` storage — an **out-of-bounds memory access**.

---

## 3. Trigger Scenario and Impact

### Triggering Conditions

The overflow requires `array.len() > usize::MAX / 8`:
- On **32-bit** (Nanvix target): `array.len() > 536,870,911` (512 MB array)
- On **64-bit**: `array.len() > 2,305,843,009,213,693,951` (practically impossible)

### Current Risk

The bitmap is primarily used by the slab allocator, where `index_len = total_num_blocks / 8`.
For `index_len` to exceed 512 MB on a 32-bit system, the slab would need
`total_num_blocks > 4 billion`, which exceeds `usize::MAX` on 32-bit. So **the overflow
cannot be triggered via the slab allocator's normal call path**.

However, `Bitmap::from_raw_array` is a public API that can be called directly. A future caller
with a large `RawArray` (e.g., a frame allocator managing all physical memory) could trigger
the overflow.

| Aspect | Rating |
|--------|--------|
| Severity | MEDIUM |
| Exploitability | Requires very large array on 32-bit systems |
| Current risk | LOW (slab parameters prevent triggering) |
| API correctness | BROKEN (public API has silent overflow) |

---

## 4. How Verus Verification Discovered This

When verifying `from_raw_array`, Verus needed to prove that `number_of_bits` (a `usize`) does
not overflow. The expression `array.len() * u8::BITS as usize` generates a proof obligation:

> `(array@.len() as int) * 8 <= usize::MAX as int`

This cannot be proven without an assumption on `array.len()`. The prover added the
precondition:

```rust
requires
    array@.len() <= usize::MAX / (u8::BITS as usize),
```

Similarly, the verified `index()` was changed to use `self.number_of_bits` instead of
recomputing `self.bits.len() * u8::BITS`, avoiding the overflow entirely.

---

## 5. The Verified Fix

### `from_raw_array` precondition ([`verus/split/libs/bitmap/lib.rs:122-127`](../verus/split/libs/bitmap/lib.rs#L122-L127))

```rust
pub fn from_raw_array(array: RawArray<u8>) -> (result: Self)
    requires
        array@.len() > 0,
        array@.len() <= usize::MAX / (u8::BITS as usize),      // ← Prevents overflow
        array@.len() * (u8::BITS as usize) < u32::MAX as usize, // ← Tighter bound
        forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0,
```

### `index()` uses stored field ([`verus/split/libs/bitmap/lib.rs:970+`](../verus/split/libs/bitmap/lib.rs#L970))

```rust
fn index(&self, bit_index: usize) -> (result: Result<(usize, usize), Error>)
{
    if bit_index >= self.number_of_bits {  // ← No multiplication, uses stored field
        return Err(...)
    }
    Ok(self.index_unchecked(bit_index))
}
```

---

## 6. Current Status in Integration Branch

The integration branch has already ported parts of the Verus bitmap:

- `from_raw_array` **no longer zeroes memory** (correct, since `RawArray::new_unmanaged`
  calls `write_bytes(0)` internally).
- `new()` **no longer has an explicit zero loop** (same reason).
- **However**, `from_raw_array` still has no runtime overflow check on `array.len() * u8::BITS`.
- The `index()` function currently uses `self.bits.len() * u8::BITS` (the original overflowing
  form), not the verified `self.number_of_bits`.

---

## 7. Proposed Fix

### For `from_raw_array`

```rust
pub fn from_raw_array(array: RawArray<u8>) -> Result<Self, Error> {
    // Check for multiplication overflow.
    if array.len() > usize::MAX / u8::BITS as usize {
        return Err(Error::new(ErrorCode::InvalidArgument, "array too large"));
    }

    Ok(Self {
        number_of_bits: array.len() * u8::BITS as usize,
        bits: array,
        usage: 0,
    })
}
```

Note: this changes the return type from `Self` to `Result<Self, Error>`, which requires
updating callers. Alternatively, add a `debug_assert!` if the API change is undesirable.

### For `index()`

```rust
fn index(&self, index: usize) -> Result<(usize, usize), Error> {
    // Use stored number_of_bits instead of recomputing to avoid overflow.
    if index >= self.number_of_bits {
        return Err(Error::new(ErrorCode::InvalidArgument, "index out of bounds"));
    }
    Ok(self.index_unchecked(index))
}
```

---

## 8. File Reference Index

| File | Role | Key Lines |
|------|------|-----------|
| `src/libs/bitmap/src/lib.rs` (dev) | **Original code (buggy)** | L112: overflow in from_raw_array; L306: overflow in index() |
| [`verus/split/libs/bitmap/lib.rs`](../verus/split/libs/bitmap/lib.rs) | **Verified code (fixed)** | L125: precondition; L970+: uses stored field |
| `src/libs/bitmap/src/lib.rs` (integration) | **Current code** | Partially ported; overflow still present |
