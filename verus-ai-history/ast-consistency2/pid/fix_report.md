# Exec Consistency Fix: pid

## Summary
- Mismatches fixed: 0 (all 5 are documented equivalences — no exec logic divergence)
- Missing functions added: 0
- Extra functions removed: 0 (all 23 are justified verification helpers)
- Documented equivalences: 28

## Structural Note

The original `ProcessIdentifier` [struct_ProcessIdentifier.diff](struct_ProcessIdentifier.diff) | [struct_ProcessIdentifier_source.rs](struct_ProcessIdentifier_source.rs) | [struct_ProcessIdentifier_verus.rs](struct_ProcessIdentifier_verus.rs) is a tuple struct (`ProcessIdentifier(i32)`) with a
`#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]` attribute.
The Verus version uses a named-field struct (`ProcessIdentifier { pub value: i32 }`)
because Verus requires named fields for spec-level reasoning (e.g., `self@.value`).
Derived trait impls are replaced with explicit impls that delegate to verified helper
methods inside the `verus!` block — Verus cannot verify trait `impl` blocks directly.

All field access changes (`self.0` → `self.value`) and constructor changes
(`ProcessIdentifier(x)` → `ProcessIdentifier { value: x }`) are mechanical
consequences of this struct representation change and are semantically identical.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `ProcessIdentifier` [struct_ProcessIdentifier.diff](struct_ProcessIdentifier.diff) | [struct_ProcessIdentifier_source.rs](struct_ProcessIdentifier_source.rs) | [struct_ProcessIdentifier_verus.rs](struct_ProcessIdentifier_verus.rs) (struct) | Documented equivalence | Tuple struct → named struct required for Verus spec reasoning (`self@.value`). Layout is identical (`#[repr(C)]`, single `i32` field). |
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | Documented equivalence | `self.0` → `self.value` due to struct change. Same `write!` format, same output. |
| `from` [from.diff](from.diff) | [from_source.rs](from_source.rs) | [from_verus.rs](from_verus.rs) (From\<i32\>) | Documented equivalence | `ProcessIdentifier(raw_tid)` → `Self::from_i32(raw)` which does `ProcessIdentifier { value: raw }`. Identical behavior. |
| `from_ne_bytes` [from_ne_bytes.diff](from_ne_bytes.diff) | [from_ne_bytes_source.rs](from_ne_bytes_source.rs) | [from_ne_bytes_verus.rs](from_ne_bytes_verus.rs) | Documented equivalence | `Self(i32::from_ne_bytes(bytes))` → `ProcessIdentifier { value: i32::from_ne_bytes(bytes) }`. Array size `core::mem::size_of::<i32>()` → literal `4` (Verus cannot evaluate const generics with `size_of`). Identical behavior. |
| `to_ne_bytes` [to_ne_bytes.diff](to_ne_bytes.diff) | [to_ne_bytes_source.rs](to_ne_bytes_source.rs) | [to_ne_bytes_verus.rs](to_ne_bytes_verus.rs) | Documented equivalence | `self.0.to_ne_bytes()` → `self.value.to_ne_bytes()`. Array size `core::mem::size_of::<i32>()` → literal `4`. Identical behavior. |
| `try_from` [try_from.diff](try_from.diff) | [try_from_source.rs](try_from_source.rs) | [try_from_verus.rs](try_from_verus.rs) (TryFrom\<u64\>) | Documented equivalence | Original uses `raw_tid.try_into().map_err(...).map(ProcessIdentifier)`; Verus delegates to `try_from_u64(raw)` which checks `raw > i32::MAX as u64`. For `u64`, `try_into::<i32>` fails iff value > `i32::MAX`, so the explicit check is equivalent. |
| `from_i32` [from_i32_verus.rs](from_i32_verus.rs) | Justified extra | Verified helper for `From<i32>::from`. Enables Verus to verify value preservation (`result@.value == raw as int`). |
| `into_i32` [into_i32_verus.rs](into_i32_verus.rs) | Justified extra | Verified helper for `From<ProcessIdentifier>::from` (i32). Verifies `result as int == self@.value`. |
| `into_isize` [into_isize_verus.rs](into_isize_verus.rs) | Justified extra | Verified helper for `From<ProcessIdentifier>::from` (isize). Verifies widening cast preserves value. |
| `into_i64` [into_i64_verus.rs](into_i64_verus.rs) | Justified extra | Verified helper for `From<ProcessIdentifier>::from` (i64). Verifies widening cast preserves value. |
| `try_into_usize` [try_into_usize_verus.rs](try_into_usize_verus.rs) | Justified extra | Verified helper for `TryFrom<ProcessIdentifier> for usize`. Explicit negativity check equivalent to `i32::try_into::<usize>()`. |
| `try_into_u32` [try_into_u32_verus.rs](try_into_u32_verus.rs) | Justified extra | Verified helper for `TryFrom<ProcessIdentifier> for u32`. Explicit negativity check equivalent to `i32::try_into::<u32>()`. |
| `try_into_u64` [try_into_u64_verus.rs](try_into_u64_verus.rs) | Justified extra | Verified helper for `TryFrom<ProcessIdentifier> for u64`. Explicit negativity check equivalent to `i32::try_into::<u64>()`. |
| `try_from_isize` [try_from_isize_verus.rs](try_from_isize_verus.rs) | Justified extra | Verified helper for `TryFrom<isize> for ProcessIdentifier`. Explicit range check equivalent to `isize::try_into::<i32>()`. |
| `try_from_i64` [try_from_i64_verus.rs](try_from_i64_verus.rs) | Justified extra | Verified helper for `TryFrom<i64> for ProcessIdentifier`. Explicit range check equivalent to `i64::try_into::<i32>()`. |
| `try_from_usize` [try_from_usize_verus.rs](try_from_usize_verus.rs) | Justified extra | Verified helper for `TryFrom<usize> for ProcessIdentifier`. Explicit `> i32::MAX` check equivalent to `usize::try_into::<i32>()`. |
| `try_from_u32` [try_from_u32_verus.rs](try_from_u32_verus.rs) | Justified extra | Verified helper for `TryFrom<u32> for ProcessIdentifier`. Explicit `> i32::MAX` check equivalent to `u32::try_into::<i32>()`. |
| `try_from_u64` [try_from_u64_verus.rs](try_from_u64_verus.rs) | Justified extra | Verified helper for `TryFrom<u64> for ProcessIdentifier`. Explicit `> i32::MAX` check equivalent to `u64::try_into::<i32>()`. |
| `eq` [eq_verus.rs](eq_verus.rs) | Justified extra | Verified helper for `PartialEq::eq`. Replaces derived `PartialEq`. Verifies `result == (self@.value == other@.value)`. |
| `ne` [ne_verus.rs](ne_verus.rs) | Justified extra | Verified helper complementing `eq` [eq_verus.rs](eq_verus.rs). Verifies `result == (self@.value != other@.value)`. |
| `lt` [lt_verus.rs](lt_verus.rs) | Justified extra | Verified comparison helper. Verifies `result == (self@.value < other@.value)`. |
| `le` [le_verus.rs](le_verus.rs) | Justified extra | Verified comparison helper. Verifies `result == (self@.value <= other@.value)`. |
| `gt` [gt_verus.rs](gt_verus.rs) | Justified extra | Verified comparison helper. Verifies `result == (self@.value > other@.value)`. |
| `ge` [ge_verus.rs](ge_verus.rs) | Justified extra | Verified comparison helper. Verifies `result == (self@.value >= other@.value)`. |
| `cmp_ord` [cmp_ord_verus.rs](cmp_ord_verus.rs) | Justified extra | Verified helper for `Ord::cmp`. Returns correct `Ordering` variant. |
| `default_value` [default_value_verus.rs](default_value_verus.rs) | Justified extra | Verified helper for `Default::default`. Verifies `result@.value == 0`. |
| `default` [default_verus.rs](default_verus.rs) (trait impl) | Justified extra | Thin wrapper delegating to `default_value()`. Replaces derived `Default`. |
| `partial_cmp` [partial_cmp_verus.rs](partial_cmp_verus.rs) (trait impl) | Justified extra | Thin wrapper delegating to `cmp()`. Replaces derived `PartialOrd`. |
| `cmp` [cmp_verus.rs](cmp_verus.rs) (trait impl) | Justified extra | Thin wrapper delegating to `cmp_ord()`. Replaces derived `Ord`. |

## Verification: PASS

```
verus --crate-type lib lib.rs --verify-module libs::sys::sys::pm::pid
verification results:: 38 verified, 0 errors
```
