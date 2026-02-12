# Bug Report: Bitmap::index() Bounds Check Uses Recomputed Overflow-Prone Expression

> **Severity:** MEDIUM
> **Status:** Confirmed in dev branch; still present in integration branch
> **Discovery Method:** Structural — Verus verification model uses stored `number_of_bits` field
> instead of recomputing `bits.len() * u8::BITS`, avoiding an overflow that can bypass bounds checking
> **Affected File:** `src/libs/bitmap/src/lib.rs` (dev branch line 306; integration branch line 1126)
> **Verified File:** [`verus/split/libs/bitmap/lib.rs`](../verus/split/libs/bitmap/lib.rs) (line 982, uses stored field)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Why This Matters](#3-why-this-matters)
4. [How Verus Verification Avoids This](#4-how-verus-verification-avoids-this)
5. [Proposed Fix](#5-proposed-fix)
6. [File Reference Index](#6-file-reference-index)

---

## 1. Executive Summary

The `index()` function is the **sole bounds-checking gateway** for all bit operations (`set`,
`clear`, `test`). It computes the bounds check as `index >= self.bits.len() * u8::BITS as usize`,
which involves a multiplication that can overflow on 32-bit systems. If the multiplication wraps,
the bounds check produces the wrong result, and `index_unchecked()` subsequently computes an
out-of-bounds word index, leading to memory access beyond the `RawArray` storage.

The Verus verified version uses `self.number_of_bits` (a pre-computed, stored field) instead of
recomputing `bits.len() * 8` on every call, completely eliminating the overflow risk.

---

## 2. The Bug

### Root Cause

In the dev branch ([`src/libs/bitmap/src/lib.rs`, line 306](../src/libs/bitmap/src/lib.rs)):

```rust
fn index(&self, index: usize) -> Result<(usize, usize), Error> {
    // Check if the index is out of bounds.
    if index >= self.bits.len() * u8::BITS as usize {  // ← Multiplication can overflow!
        let reason: &str = "index out of bounds";
        return Err(Error::new(ErrorCode::InvalidArgument, reason));
    }
    Ok(self.index_unchecked(index))
}
```

The same pattern persists in the current integration branch at line 1126.

### What Happens on Overflow

If `self.bits.len() = 0x3000_0000` (805 MB) on a 32-bit system:

```
self.bits.len() * 8 = 0x3000_0000 * 8 = 0x1_8000_0000
Wraps to: 0x8000_0000 (2,147,483,648)
```

Now the bounds check is:
```rust
if index >= 0x8000_0000  // Wrong! Should be 0x1_8000_0000
```

- Indices from `0x8000_0000` to `0x1_7FFF_FFFF` **pass** the bounds check (false negative).
- `index_unchecked` computes `word = index / 8`, which can exceed `self.bits.len()`.
- The subsequent `self.bits[word]` accesses memory **beyond** the `RawArray` — undefined behavior.

### Impact Chain

Every `set`, `clear`, and `test` call flows through `index()`:

```
set(index) → index(index) → [OVERFLOW → bounds check bypassed] → index_unchecked(index) → OOB access
clear(index) → index(index) → [same]
test(index) → index(index) → [same]
```

This makes `index()` a **critical security boundary**: if its bounds check fails, all bitmap
operations become unsafe.

---

## 3. Why This Matters

### Defense in Depth

Even if `from_raw_array` is fixed to prevent overflow in `number_of_bits`, the `index()`
function independently recomputes `bits.len() * 8`. This means:

1. A fix in `from_raw_array` does **not** fix `index()`.
2. The `index()` overflow is independently exploitable.
3. Having two separate computations of the same value (`number_of_bits` and `bits.len() * 8`)
   violates the DRY principle and creates inconsistency risk.

### Relationship to `number_of_bits` Field

The bitmap struct already stores `number_of_bits`. The recomputation in `index()` is
**redundant** — the stored field exists precisely to avoid this. Using the stored field
is both safer (no overflow risk) and faster (no multiplication on every call).

---

## 4. How Verus Verification Avoids This

The verified `index()` ([`verus/split/libs/bitmap/lib.rs`](../verus/split/libs/bitmap/lib.rs))
uses the stored field:

```rust
fn index(&self, bit_index: usize) -> (result: Result<(usize, usize), Error>)
    requires self.inv(),
{
    if bit_index >= self.number_of_bits {  // ← Stored field, no multiplication
        let reason: &str = "index out of bounds";
        return Err(Error::new(ErrorCode::InvalidArgument, reason));
    }
    Ok(self.index_unchecked(bit_index))
}
```

The invariant `self.inv()` guarantees `self.number_of_bits == self.bits@.len() * 8`,
so the two expressions are equivalent **when the invariant holds**. But using the stored
field is inherently safer because it avoids the overflow entirely.

---

## 5. Proposed Fix

Replace the recomputation with the stored field:

```rust
fn index(&self, index: usize) -> Result<(usize, usize), Error> {
    // Use stored number_of_bits to avoid multiplication overflow.
    if index >= self.number_of_bits {
        let reason: &str = "index out of bounds";
        return Err(Error::new(ErrorCode::InvalidArgument, reason));
    }
    Ok(self.index_unchecked(index))
}
```

This is a **one-line change** with zero behavioral impact under normal conditions.

---

## 6. File Reference Index

| File | Role | Key Lines |
|------|------|-----------|
| `src/libs/bitmap/src/lib.rs` (dev) | **Original code (buggy)** | L306: `self.bits.len() * u8::BITS` overflow |
| `src/libs/bitmap/src/lib.rs` (integration) | **Current code (same bug)** | L1126: same pattern |
| [`verus/split/libs/bitmap/lib.rs`](../verus/split/libs/bitmap/lib.rs) | **Verified code (fixed)** | Uses `self.number_of_bits` |
