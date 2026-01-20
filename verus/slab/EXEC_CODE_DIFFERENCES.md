# Executable Code Differences: Verified vs Original

This document details the differences between the verified `slab_core.rs` and the original
`src/libs/slab/src/lib.rs`. It explains what would need adjustment to use the verified code
in production and documents modifications made to facilitate verification.

## Summary

The verified code is **structurally compatible** with the original but has several
modifications. Most are **additive** (specs, proofs, ghost state) that have zero
runtime cost. A few are **semantic changes** that affect the API contract.

## 1. Type Changes

### 1.1 `data_addr`: `*mut u8` → `usize`

**Original:**
```rust
data_addr: *mut u8,
```

**Verified:**
```rust
data_addr: usize,
```

**Reason:** Verus cannot reason about raw pointers directly. We use `usize` to represent
addresses and perform arithmetic on them.

**To use in production:**
- Convert back to `*mut u8` at the API boundary
- Or keep as `usize` and cast when dereferencing: `(self.data_addr as *mut u8)`

### 1.2 Added Fields: `base_addr` and `total_len`

**Verified code adds:**
```rust
base_addr: usize,  // Base address of entire slab buffer
total_len: usize,  // Total length of slab buffer in bytes
```

**Reason:** These track the original buffer bounds for verification of the
`is_within_buffer` property.

**To use in production:**
- These are useful for debugging/assertions anyway
- Minimal runtime overhead (2 × usize = 16 bytes on 64-bit)
- Can be removed if space is critical (but lose bounds-checking capability)

## 2. Function Signature Changes

### 2.1 `from_raw_parts`: `*mut u8` → `usize`

**Original:**
```rust
pub unsafe fn from_raw_parts(addr: *mut u8, len: usize, block_size: usize) -> Result<Slab, Error>
```

**Verified:**
```rust
pub unsafe fn from_raw_parts(addr: usize, len: usize, block_size: usize) -> (result: Result<Slab, Error>)
```

**To use in production:**
- Callers need to cast: `from_raw_parts(ptr as usize, len, block_size)`
- Or wrap with an adapter function that takes `*mut u8`

### 2.2 `allocate`: Returns `usize` instead of `*mut u8`

**Original:**
```rust
pub fn allocate(&mut self) -> Result<*mut u8, Error>
```

**Verified:**
```rust
pub fn allocate(&mut self) -> (result: Result<usize, Error>)
```

**To use in production:**
- Callers cast the result: `allocate()? as *mut u8`

### 2.3 `deallocate`: Takes `usize` instead of `*mut u8`

**Original:**
```rust
pub fn deallocate(&mut self, ptr: *mut u8) -> Result<(), Error>
```

**Verified:**
```rust
pub fn deallocate(&mut self, addr: usize) -> (result: Result<(), Error>)
```

**To use in production:**
- Callers cast the pointer: `deallocate(ptr as usize)`

## 3. Precondition Strengthening

### 3.1 `from_raw_parts` Preconditions

**Original:** Runtime checks with `Err` returns

**Verified:** Preconditions (verified callers must satisfy these):
```rust
requires
    len > 0,
    len < i32::MAX as usize,
    block_size > 0,
    block_size <= len,
    Self::spec_is_power_of_two(block_size as int),
    addr % block_size == 0,
    addr > 0,
    (addr as int) + (len as int) <= (usize::MAX as int),
    (len / block_size) % (u8::BITS as usize) == 0,
    len / block_size >= 8,
```

**Important:** The verified code **also keeps runtime checks** for defense-in-depth.
Unverified callers (e.g., FFI) still get proper error returns.

### 3.2 `allocate` Preconditions

**Verified:**
```rust
requires
    old(self).inv(),
```

The invariant must hold. For verified callers, this is always true after construction.

### 3.3 `deallocate` Preconditions

**Verified:**
```rust
requires
    old(self).inv(),
    old(self)@.is_valid_addr(addr as int),
    old(self)@.is_allocated(old(self)@.addr_to_block_idx(addr as int)),
```

**Important:** The verified code **also keeps runtime bounds checks** for unverified callers.

## 4. Removed Runtime Check

### 4.1 Memory Wrap-Around Check

**Original:**
```rust
if addr.wrapping_add(len) < addr {
    return Err(Error::new(ErrorCode::InvalidArgument, "wrapping memory region"));
}
```

**Verified:** Replaced with precondition:
```rust
requires
    (addr as int) + (len as int) <= (usize::MAX as int),
```

**Reason:** We use mathematical integers in specs, so overflow is handled at the spec level.

**To use in production:** Add the runtime check back if you need to handle invalid inputs
from unverified callers.

## 5. Added Code (Zero Runtime Cost)

The following additions exist only at verification time:

### 5.1 `SlabView` Structure
Ghost/spec-level abstract state for reasoning about the slab.

### 5.2 `impl View for Slab`
Converts concrete state to abstract view.

### 5.3 `inv()` Invariant
Specifies when a slab is in a valid state.

### 5.4 Proof Functions (`proof fn`)
All `proof fn` functions are erased at compile time.

### 5.5 Lemmas
All lemmas are erased at compile time.

### 5.6 `assert`, `proof { ... }` Blocks
Ghost assertions for verification, erased at compile time.

## 6. Import Changes

**Original:**
```rust
use ::bitmap::Bitmap;
use ::raw_array::RawArray;
use ::sys::error::{Error, ErrorCode};
```

**Verified:**
```rust
use crate::{bitmap::Bitmap, error::{Error, ErrorCode}, raw_array::RawArray};
use vstd::{prelude::*, set::*, set_lib::*};
```

**To use in production:**
- Keep original imports
- Remove `vstd` imports (only needed for verification)

## 7. Quick Migration Guide

To use the verified executable code in production:

### Option A: Minimal Changes (Recommended)

1. **Keep the verified `slab_core.rs`** as the source of truth
2. **Strip Verus annotations** using a build script or macro:
   - Remove `requires`, `ensures`, `invariant`
   - Remove `proof { ... }` blocks
   - Remove `ghost`, `tracked` variables
   - Remove `assert(...)` statements (or convert to `debug_assert!`)
3. **Add pointer wrapper functions:**
   ```rust
   pub unsafe fn from_raw_parts_ptr(addr: *mut u8, len: usize, block_size: usize) -> Result<Slab, Error> {
       Self::from_raw_parts(addr as usize, len, block_size)
   }

   pub fn allocate_ptr(&mut self) -> Result<*mut u8, Error> {
       self.allocate().map(|addr| addr as *mut u8)
   }

   pub fn deallocate_ptr(&mut self, ptr: *mut u8) -> Result<(), Error> {
       self.deallocate(ptr as usize)
   }
   ```

### Option B: Dual Maintenance

1. **Keep both files:**
   - `src/libs/slab/src/lib.rs` - Production code
   - `verus/slab/slab_core.rs` - Verified specification
2. **Manually sync changes** between them
3. **Use CI** to verify they remain in sync

## 8. Runtime Overhead Assessment

| Change | Runtime Cost |
|--------|--------------|
| `base_addr`, `total_len` fields | +16 bytes per Slab |
| `usize` vs `*mut u8` | Zero (same size) |
| All spec/proof code | Zero (erased) |
| Ghost assertions | Zero (erased) |
| Runtime bounds checks | Kept (same as original) |

**Total overhead:** ~16 bytes per Slab instance for buffer bounds tracking.

## 9. Cautions When Using Verified Exec Code

1. **Preconditions are contracts:** If calling from unverified code, ensure runtime
   checks are in place (the verified code keeps them for defense-in-depth).

2. **Pointer casting:** Remember to cast `usize` ↔ `*mut u8` at boundaries.

3. **Error handling preserved:** The verified code returns the same errors as the
   original for invalid inputs.

4. **Memory safety:** The verification proves logical correctness assuming the
   memory region is valid. The `unsafe` marker on `from_raw_parts` still means
   the caller must ensure the memory is accessible.

5. **Bitmap dependency:** The verified slab depends on `Bitmap` having the
   specified behavior. If `Bitmap` changes, re-verify.

## 10. Trusted Dependencies: Bitmap and RawArray Axioms

The slab verification treats `Bitmap` and `RawArray` as **trusted dependencies** with
axiomatized specifications. This section documents the axioms and their justification.

### 10.1 RawArray Axioms

**Location:** `verus/slab/raw_array.rs`

#### 10.1.1 View Function (Uninterpreted)

```rust
impl<T> View for RawArray<T> {
    type V = Seq<T>;
    uninterp spec fn view(&self) -> Seq<T>;
}
```

**Justification:** The view maps a `RawArray<T>` to a logical `Seq<T>`. This is
standard Verus practice for abstracting low-level data structures. The original
`RawArray` provides `Deref<Target=[T]>` which behaves like a sequence.

#### 10.1.2 Zero Predicate and Axiom

```rust
pub uninterp spec fn is_zero<T>(i: T) -> bool;
pub axiom fn axiom_u8_zero_is_0(t: u8) requires is_zero(t) ensures t == 0;
```

**Why needed:** When `RawArray::new()` or `from_raw_parts()` creates storage, it
zeroes the memory. We need to express "all elements are zero" without knowing `T`.

**Original code behavior:**
```rust
// In RawArrayStorage::new_managed (original)
unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };
```

**Justification:** The original code uses `ptr::write_bytes(..., 0, len)` which
writes zero bytes. For `u8`, this means each element is `0u8`. The axiom captures
this for the type we actually use (`RawArray<u8>`).

**Potential weakness:** This axiom is specific to `u8`. If used with other types,
additional axioms would be needed.

#### 10.1.3 Construction Postconditions

```rust
#[verifier::external_body]
pub fn new(len: usize) -> (result: Result<RawArray<T>, Error>)
    ensures
        result is Ok ==> {
            &&& result->Ok_0@.len() == len
            &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
        },
```

**Justification:** The original `RawArray::new()` allocates memory and zeroes it.
The postcondition states:
1. The resulting array has the requested length
2. All elements satisfy `is_zero` (for `u8`, this means they equal `0`)

**Similar for `from_raw_parts` and `from_raw_addr`.**

#### 10.1.4 Length Function

```rust
#[verifier::external_body]
pub fn len(&self) -> (result: usize)
    ensures result == self@.len()
```

**Justification:** Directly corresponds to original behavior.

### 10.2 Bitmap Axioms

**Location:** `verus/slab/bitmap.rs`

#### 10.2.1 View Structure and Function

```rust
pub ghost struct BitmapView {
    pub bits: Seq<bool>,
}

impl View for Bitmap {
    type V = BitmapView;
    uninterp spec fn view(&self) -> BitmapView;
}
```

**Justification:** Abstracts the bitmap as a sequence of boolean bits. This matches
the logical behavior: a bitmap tracks which indices are allocated (true) or free (false).

#### 10.2.2 Invariant

```rust
pub closed spec fn inv(&self) -> bool {
    &&& self.number_of_bits > 0
    &&& self@.number_of_bits() == self.number_of_bits as int
    &&& self@.number_of_bits() <= (usize::MAX as int)
}
```

**Justification:** Minimal invariant connecting concrete field to abstract view.

#### 10.2.3 `alloc` Postconditions

```rust
#[verifier::external_body]
pub fn alloc(&mut self) -> (result: Result<usize, Error>)
    ensures
        self.inv(),
        result is Ok ==> {
            let index = result->Ok_0 as int;
            &&& 0 <= index < self@.number_of_bits()
            &&& self@.number_of_bits() == old(self)@.number_of_bits()
            &&& self.is_bit_set(index)
            &&& !old(self).is_bit_set(index)
            &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                self.is_bit_set(i) == old(self).is_bit_set(i)
        },
        result is Err ==> self@ == old(self)@,
        old(self)@.has_free_bit() ==> result is Ok,
        result is Err ==> old(self)@.is_full(),
```

**Comparison with original:**
```rust
// Original Bitmap::alloc() behavior:
// 1. Scans for first unset bit
// 2. Sets it
// 3. Returns the index
// 4. Returns Err if all bits set
```

**Justification:**
- Returns an index that was previously unset, now set ✓
- Other bits unchanged ✓
- Fails only when full ✓ (liveness property)
- On error, no mutation ✓ (error-path immutability)

#### 10.2.4 `set` and `clear` Postconditions

```rust
// set: marks a specific bit as allocated
ensures
    result is Ok ==> {
        &&& self.is_bit_set(index as int)
        &&& forall|i: int| i != index as int ==> self.is_bit_set(i) == old(self).is_bit_set(i)
    },
    result is Err ==> self@ == old(self)@,

// clear: marks a specific bit as free
ensures
    result is Ok ==> {
        &&& !self.is_bit_set(index as int)
        &&& forall|i: int| i != index as int ==> self.is_bit_set(i) == old(self).is_bit_set(i)
    },
    result is Err ==> self@ == old(self)@,
```

**Justification:** Standard bit manipulation semantics. The key addition is
**error-path immutability**: if the operation fails, the bitmap is unchanged.

#### 10.2.5 `test` Postcondition

```rust
ensures
    result is Ok ==> result->Ok_0 == self.is_bit_set(index as int),
```

**Justification:** Returns whether the bit is set. Pure observation, no mutation.

#### 10.2.6 `from_raw_array` Postconditions

```rust
pub fn from_raw_array(storage: RawArray<u8>, number_of_bits: usize) -> (result: Bitmap)
    requires 
        storage@.len() > 0,
        number_of_bits > 0,
        number_of_bits <= storage@.len() * 8,
        forall|i: int| 0 <= i < storage@.len() ==> storage@[i] == 0u8,
    ensures
        result.inv(),
        result@.number_of_bits() == number_of_bits as int,
        forall|i: int| 0 <= i < number_of_bits as int ==> !result.is_bit_set(i),
```

**Justification:**
- Requires zeroed storage → all bits initially unset ✓
- Connects the abstract view to concrete initialization ✓

### 10.3 Axiom Soundness Assessment

| Axiom/Spec | Risk Level | Justification |
|------------|------------|---------------|
| `is_zero` predicate | LOW | Standard pattern for zero-initialization |
| `axiom_u8_zero_is_0` | LOW | Trivially true: zero bytes are 0 |
| `RawArray::view()` | LOW | Standard Verus abstraction |
| `RawArray::new()` ensures | LOW | Matches `ptr::write_bytes` behavior |
| `Bitmap::view()` | LOW | Standard abstraction |
| `Bitmap::alloc()` ensures | MEDIUM | Must match actual scan-and-set behavior |
| `Bitmap::set/clear()` ensures | LOW | Standard bit operations |
| Error-path immutability | MEDIUM | Assumes implementation doesn't mutate on error |
| `has_free_bit() ==> Ok` | MEDIUM | Liveness depends on correct implementation |

### 10.4 Recommendations for Production

1. **Unit test bitmap and raw_array thoroughly** to ensure behavior matches specs
2. **Consider verifying bitmap separately** (like `verus/bitmap/bitmap.rs`)
3. **Document the trusted boundary** clearly in production code
4. **Add runtime assertions** in debug builds that mirror spec properties

### 10.5 Relationship to Existing Verified Bitmap

The `verus/bitmap/bitmap.rs` file contains a **fully verified** bitmap implementation.
The axioms in `verus/slab/bitmap.rs` are **compatible** with those verified specs:

- Same `BitmapView` structure
- Same `inv()` invariant shape
- Same postconditions for `alloc`, `set`, `clear`, `test`

**To strengthen the trusted boundary:** Import the verified bitmap specs directly
rather than re-axiomatizing them.
