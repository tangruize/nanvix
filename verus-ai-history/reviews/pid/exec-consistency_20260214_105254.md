# Review: pid Exec Consistency (claude-opus-4.6)

## Grade: A

## Summary

The exec consistency fix for `ProcessIdentifier` is thorough, well-documented, and
verification-clean (38 verified, 0 errors). Every original function has a verified
counterpart, and structural changes are necessitated by Verus limitations. The
equivalence justifications are sound. One minor concern prevents an A+.

## Verification Status

**PASS**: 38 verified, 0 errors.

## Completeness Check

### Original Functions → Verus Coverage

| Original | Verus Equivalent | Status |
|----------|-----------------|--------|
| `ProcessIdentifier(i32)` tuple struct | `ProcessIdentifier { value: i32 }` named struct | ✅ Equivalent (`#[repr(C)]` single-field, identical layout) |
| `KERNEL_RAW = 0` | `KERNEL_RAW = 0` | ✅ Identical |
| `KERNEL` | `KERNEL` | ✅ Equivalent (constructor syntax differs) |
| `PARSE_ERROR_MESSAGE` | `PARSE_ERROR_MESSAGE` | ✅ Identical |
| `INITD` | `INITD` | ✅ Equivalent (constructor syntax differs) |
| `to_ne_bytes` | `to_ne_bytes` | ✅ Equivalent (`external_body`, justified) |
| `from_ne_bytes` | `from_ne_bytes` | ✅ Equivalent (`external_body`, justified) |
| `From<ProcessIdentifier> for isize` | `into_isize` + trait wrapper | ✅ Equivalent |
| `From<ProcessIdentifier> for i32` | `into_i32` + trait wrapper | ✅ Equivalent |
| `From<ProcessIdentifier> for i64` | `into_i64` + trait wrapper | ✅ Equivalent |
| `TryFrom<ProcessIdentifier> for usize` | `try_into_usize` + trait wrapper | ✅ Equivalent |
| `TryFrom<ProcessIdentifier> for u32` | `try_into_u32` + trait wrapper | ✅ Equivalent |
| `TryFrom<ProcessIdentifier> for u64` | `try_into_u64` + trait wrapper | ✅ Equivalent |
| `TryFrom<isize> for ProcessIdentifier` | `try_from_isize` + trait wrapper | ✅ Equivalent |
| `From<i32> for ProcessIdentifier` | `from_i32` + trait wrapper | ✅ Equivalent |
| `TryFrom<i64> for ProcessIdentifier` | `try_from_i64` + trait wrapper | ✅ Equivalent |
| `TryFrom<usize> for ProcessIdentifier` | `try_from_usize` + trait wrapper | ✅ Equivalent |
| `TryFrom<u32> for ProcessIdentifier` | `try_from_u32` + trait wrapper | ✅ Equivalent |
| `TryFrom<u64> for ProcessIdentifier` | `try_from_u64` + trait wrapper | ✅ Equivalent |
| `Debug::fmt` | `Debug::fmt` | ✅ Equivalent (`self.0` → `self.value`) |
| `static_assert_eq_size!(4)` | `lemma_size_eq_i32` (`external_body`) | ✅ Equivalent |
| `static_assert_eq_align!(4)` | `lemma_align_eq_i32` (`external_body`) | ✅ Equivalent |
| Derived `Default` | `default_value` + `Default` trait wrapper | ✅ Equivalent |
| Derived `Clone, Copy` | `#[derive(Clone, Copy)]` | ✅ Identical |
| Derived `PartialEq, Eq` | `eq`/`ne` + `PartialEq`/`Eq` wrappers | ✅ Equivalent |
| Derived `PartialOrd, Ord` | `lt`/`le`/`gt`/`ge`/`cmp_ord` + trait wrappers | ✅ Equivalent |

**No missing functions. No unaccounted extras.**

## Issues Found

### Critical

- None.

### Minor

1. **Visibility widening of `PARSE_ERROR_MESSAGE`**: The original declares
   `const PARSE_ERROR_MESSAGE` (private, no `pub`), while the Verus version uses
   `pub const PARSE_ERROR_MESSAGE`. This is a minor visibility difference. It does
   not affect correctness, but it expands the public API surface. Since the trait
   wrappers outside `verus!` need to reference it, this is understandable, though
   the verified helpers already handle error construction internally, making the
   `pub` on this constant technically unnecessary for the trait impls.

### Observations (Non-Issues)

1. **`value` field is `pub`**: The original uses a private tuple field `self.0`.
   The Verus version makes `pub value: i32` because Verus spec reasoning requires
   named field access in `view()`. This is documented and is a known Verus limitation.
   The trait wrappers outside `verus!` do not expose raw field access to callers.

2. **TryFrom semantic equivalence**: The original uses `raw.try_into().map_err(...).map(ProcessIdentifier)`
   which chains stdlib `TryInto` conversions. The Verus version uses explicit range
   checks (e.g., `raw > i32::MAX as u64`). These are semantically equivalent because:
   - For unsigned→i32: `try_into` fails iff value > `i32::MAX`.
   - For signed→i32: `try_into` fails iff value outside `[i32::MIN, i32::MAX]`.
   - For i32→unsigned: `try_into` fails iff value < 0.
   The explicit checks make the failure conditions verifiable by Verus.

3. **Array size**: `core::mem::size_of::<i32>()` → literal `4` in byte array types.
   Required because Verus cannot evaluate `size_of` in const generic positions.
   Equivalent since `size_of::<i32>() == 4` is guaranteed by Rust.

4. **`external_body` usage**: Used in 5 places — `to_ne_bytes`, `from_ne_bytes`,
   `axiom_byte_roundtrip`, `axiom_decode_encode_roundtrip`, `axiom_from_ne_bytes_in_range`,
   `lemma_size_eq_i32`, `lemma_align_eq_i32` (7 total). All are justified by Rust's
   guaranteed semantics that Verus cannot reason about natively. The axioms form a
   consistent theory (encode-decode and decode-encode roundtrips + range constraint).

5. **Proof quality**: The proof file contains useful lemmas beyond mere consistency —
   ordering transitivity, totality, byte roundtrip composition, and view equality.
   These strengthen the verification without adding unjustified assumptions.

## Equivalence Justification Soundness

All 28 documented equivalences in the consistency report are sound:
- Struct representation change is mechanical and layout-preserving.
- All trait impls delegate to verified helpers with correct semantics.
- Range check logic in `try_from_*`/`try_into_*` matches stdlib `TryInto` behavior.
- No `assume()` or `admit()` used anywhere.

## Conclusion

The Verus exec code faithfully represents the original source. All original functions
are covered, structural changes are well-justified by Verus limitations, and
verification passes cleanly. The consistency report is accurate and thorough.
