# Review: tid Exec Consistency (claude-opus-4.6)

## Grade: A

## Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The consistency report identifies 5 functions with syntactic differences
between original and Verus versions. All 5 are documented as equivalences rather
than mismatches, which is correct:

- **`fmt`**: `self.0` → `self.value` — trivially equivalent (tuple vs named field).
- **`from` (i32 → ThreadIdentifier)**: `ThreadIdentifier(raw_tid)` → `Self::from_i32(raw)` —
  semantically identical; delegation to verified helper is the standard Verus pattern.
- **`from_ne_bytes`**: Tuple constructor → named-field constructor; `core::mem::size_of::<i32>()`
  → literal `4`. Both are correct and necessary for Verus.
- **`to_ne_bytes`**: Same field-access and array-size changes as `from_ne_bytes`.
- **`try_from` (u64)**: Opaque `try_into().map_err().map()` chain → explicit range check.
  Equivalence argument is sound (u64 ≥ 0, so only upper bound matters).

No MISMATCH entries remain unaddressed.

### 2. Were MISSING functions added with proper verification?

**N/A.** The report states 0 missing functions were added. This is consistent with
the original source — all original functions have corresponding Verus implementations.
The 23 "extra" functions are verified helpers and trait impl wrappers, not gap-fills.

### 3. Are equivalence justifications sound?

**Yes, all 5 equivalence justifications are sound:**

- The tuple→named-field struct change is well-justified. Verus requires named fields
  for `self.value as int` spec reasoning. The `#[repr(C)]` layout is preserved.
- The `core::mem::size_of::<i32>()` → `4` substitution is correct (i32 is always 4 bytes).
- The `try_into()` → explicit range check equivalences are carefully argued per category:
  - Unsigned→signed: `u64::try_into::<i32>()` fails iff `value > i32::MAX` (lower bound trivially satisfied). ✓
  - Signed→signed: explicit `< i32::MIN || > i32::MAX` check mirrors `try_into()`. ✓
  - Signed→unsigned: `< 0` check mirrors `i32::try_into::<usize>()`. ✓

### 4. Does the exec code now faithfully represent the original source?

**Yes, with minor observations:**

- All original constants preserved: `KERNEL_RAW = 0`, `KERNEL`, `INITD`.
- All original trait implementations faithfully reproduced via delegation pattern:
  `From<i32>`, `From<ThreadIdentifier> for {i32, isize, i64}`,
  `TryFrom<{isize, i64, usize, u32, u64}>`, `TryFrom<ThreadIdentifier> for {usize, u32, u64}`,
  `Debug`.
- Error messages match: `"invalid thread identifier"` throughout (via `PARSE_ERROR_MESSAGE` constant).
- Error codes match: `ErrorCode::InvalidArgument` for all conversion failures.
- The `value` field is `pub` in Verus (originally private tuple field). This is a
  visibility widening needed for spec reasoning. The trait-based API surface is
  unchanged, so this does not alter the public contract for exec consumers.

**One minor note:** The original derives `PartialEq, Eq, PartialOrd, Ord` via
`#[derive(...)]` which delegates to the i32 field's implementations. The Verus
version implements these manually via verified helpers (`eq`, `lt`, `le`, `gt`,
`ge`, `cmp_ord`). The manual implementations are semantically identical to the
derived ones — they directly compare `self.value` with `other.value` using the
same i32 operators that `derive` would generate. This is correct.

### 5. Does verification still pass?

**Yes.** Verification passes with `38 verified, 0 errors`.

## Issues Found

### Critical

- None.

### Minor

- **Field visibility widening**: `pub value: i32` vs original private `(i32)`. This is
  necessary for Verus and documented. However, downstream Verus modules could bypass
  the constructor and directly construct `ThreadIdentifier { value: ... }` without going
  through the verified `from_i32` path. The `inv()` is trivially `true`, so this has no
  safety impact, but it's worth noting for types with non-trivial invariants.

- **`PARSE_ERROR_MESSAGE` constant**: Added as a new constant not in the original. This
  is a minor code hygiene improvement (DRY principle) and does not affect semantics.
  The original inlines `"invalid thread identifier"` at each call site.

- **`ne` method**: Added but not used by any trait impl. The original derives `PartialEq`
  which provides `ne` via default method (negation of `eq`). The Verus version adds an
  explicit `ne` helper but the `PartialEq` trait impl only delegates `eq`. This is
  harmless but creates a dead-code path in the verified layer.

### Informational

- The `external_body` annotations on `to_ne_bytes`, `from_ne_bytes`, layout lemmas, and
  byte round-trip axioms are well-justified and clearly documented. The trust boundary
  is appropriately narrow.

- The spec layer (`tid.spec.rs`) cleanly separates abstraction (`ThreadIdentifierView`
  with `int` domain) from implementation. The `View` and `inv()` are both `closed`,
  following the methodology.

- The proof layer (`tid.proof.rs`) provides useful lemmas (constant properties, ordering
  consistency, byte round-trip composites, layout assertions) without any `admit()` or
  unjustified `assume()`.

## Summary

The tid exec consistency fix is thorough and well-executed. All 5 documented equivalences
are sound, the 23 extra verified helpers follow the standard Verus trait delegation pattern,
and verification passes cleanly (38/0). The exec code faithfully represents the original
source with only the minimum structural changes required by Verus (tuple→named struct,
const-generic→literal, opaque-stdlib→explicit-check). The consistency report is detailed
and accurate. Grade A — no critical issues, only minor observations about field visibility
and unused `ne` helper.
