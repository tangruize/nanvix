# Exec Integrity: process_capability

## Summary
- Total differences: 13
- Acceptable (ghost annotations): 9
- Type changes (documented): 2
- Invented functions (justified): 2
- Fixed: 0
- Unfixable (documented): 0

## Files Compared
- **Original**: `src/kernel/src/pm/process/capability.rs`
- **Verified exec**: `verus/split/kernel/pm/process/capability.rs`
- **Spec**: `verus/split/kernel/pm/process/capability.spec.rs`
- **Proof**: `verus/split/kernel/pm/process/capability.proof.rs`

## Differences

| # | Type | Location | Description | Action |
|---|------|----------|-------------|--------|
| 1 | GHOST_ANNOTATION | Module level | Added extensive module-level documentation describing abstraction levels, verified properties, and trust boundary | Acceptable |
| 2 | GHOST_ANNOTATION | Imports | Changed `use ::sys::pm::Capability` to `use crate::kernel::pm::sys::capability::Capability` + `use vstd::prelude::*` + `include!` for spec/proof files; structural adaptation for Verus verification crate | Acceptable |
| 3 | TYPE_CHANGE | `struct Capabilities` | Changed from tuple struct `Capabilities(u8)` (private field) to named struct `Capabilities { pub bits: u8 }` (public field) | Documented: Verus requires `pub` field for `pub open spec fn` definitions; field expressions must be well-formed at all visibility scopes. `pub(crate)` and `pub(super)` both rejected by Verus. Follows established pattern (cf. `ProcessIdentifier.value`). |
| 4 | TYPE_CHANGE | `struct Capabilities` derives | Changed from `#[derive(Default, Clone, Copy)]` to `#[derive(Clone, Copy, PartialEq, Eq)]` with manual `Default` impl | Documented: `PartialEq`/`Eq` required by Verus for equality reasoning. `Default` moved to manual impl to attach verification `ensures` clauses. |
| 5 | GHOST_ANNOTATION | `impl Capabilities` | Added `pub open spec fn spec_default()` | Acceptable: spec-only function, no exec code |
| 6 | INVENTED_FUNCTION | `impl Capabilities` | Added `fn to_mask(capability: Capability) -> u8` — private exec helper using explicit match instead of `1 << capability as u8` | Justified: Internal helper function (not `pub`). Produces identical mask values (1, 2, 4, 8, 16 = 2^0..2^4). Equivalence to original shift-based formula proven by `lemma_mask_matches_discriminant`. Avoids dependence on enum layout/`#[repr]` annotations. |
| 7 | INVENTED_FUNCTION | `impl Capabilities` | Added `pub fn new() -> Capabilities` constructor | Justified: Provides a verified constructor with `wf()` postcondition. Supplements (does not replace) `Default::default()`. Exec body is simply `Capabilities { bits: 0u8 }`. |
| 8 | GHOST_ANNOTATION | `fn set()` | Added `ensures` clause and `proof` block. Exec body changed from `self.0 \|= 1 << capability as u8` to `let mask: u8 = Self::to_mask(capability); self.bits = self.bits \| mask;` | Acceptable: `\|=` is semantically identical to `= ... \| ...`; `to_mask(capability)` produces the same value as `1 << capability as u8` (proven). Field access `self.0` → `self.bits` follows struct change (#3). |
| 9 | GHOST_ANNOTATION | `fn clear()` | Added `ensures` clause and `proof` block. Exec body changed from `self.0 &= !(1 << capability as u8)` to `let mask: u8 = Self::to_mask(capability); self.bits = self.bits & !mask;` | Acceptable: Same analysis as #8 — `&=` equivalent to `= ... & ...`; mask proven equivalent. |
| 10 | GHOST_ANNOTATION | `fn has()` | Added `ensures` clause. Exec body changed from `(self.0 & (1 << capability as u8)) != 0` to `let mask: u8 = Self::to_mask(capability); (self.bits & mask) != 0u8` | Acceptable: Same analysis as #8 — mask proven equivalent, field access follows struct change. |
| 11 | GHOST_ANNOTATION | `impl Default` | Replaced `#[derive(Default)]` with manual `impl Default for Capabilities` containing `ensures` clauses and `proof` block. Exec body: `Capabilities { bits: 0u8 }` | Acceptable: The derived `Default` for a struct with a single `u8` field produces `u8::default()` = `0u8`, which is identical to the manual impl's exec body. |
| 12 | GHOST_ANNOTATION | Module level | All code wrapped in `verus! {}` macro | Acceptable: Required by Verus tooling |
| 13 | GHOST_ANNOTATION | Module level | Added `include!("capability.spec.rs")` and `include!("capability.proof.rs")` | Acceptable: Brings in spec functions and proof lemmas only; no exec code |

## Exec Logic Equivalence Analysis

### `set(&mut self, capability: Capability)`

| Aspect | Original | Verified | Equivalent? |
|--------|----------|----------|-------------|
| Mask computation | `1 << capability as u8` | `Self::to_mask(capability)` via explicit match | ✅ Proven by `lemma_mask_matches_discriminant` |
| Bitwise operation | `self.0 \|= mask` | `self.bits = self.bits \| mask` | ✅ Semantically identical |
| Side effects | Mutates `self` | Mutates `self` | ✅ |

### `clear(&mut self, capability: Capability)`

| Aspect | Original | Verified | Equivalent? |
|--------|----------|----------|-------------|
| Mask computation | `1 << capability as u8` | `Self::to_mask(capability)` via explicit match | ✅ Proven |
| Bitwise operation | `self.0 &= !mask` | `self.bits = self.bits & !mask` | ✅ Semantically identical |
| Side effects | Mutates `self` | Mutates `self` | ✅ |

### `has(&self, capability: Capability) -> bool`

| Aspect | Original | Verified | Equivalent? |
|--------|----------|----------|-------------|
| Mask computation | `1 << capability as u8` | `Self::to_mask(capability)` via explicit match | ✅ Proven |
| Test expression | `(self.0 & mask) != 0` | `(self.bits & mask) != 0u8` | ✅ Identical |
| Return value | `bool` | `bool` | ✅ |

### `Default::default() -> Capabilities`

| Aspect | Original | Verified | Equivalent? |
|--------|----------|----------|-------------|
| Construction | `#[derive(Default)]` → `Capabilities(0u8)` | `Capabilities { bits: 0u8 }` | ✅ Identical initial value |

## Verification Status
- Before: **PASS** (97 verified, 0 errors)
- After: **PASS** (no changes needed)

## Notes

1. **No `assume`, `admit`, or unjustified `external_body`** found in exec, spec, or proof files.
2. **No LOGIC_CHANGE or MISSING_FUNCTION** differences were found. All three original functions (`set`, `clear`, `has`) are present with semantically equivalent exec logic.
3. The `to_mask` helper and `new` constructor are the only invented exec functions. Both are justified:
   - `to_mask`: private helper with formally proven equivalence to the original shift expression.
   - `new`: public constructor providing verified initialization; does not replace `Default`.
4. The `pub bits` field is the most significant structural deviation. It is extensively documented and follows the established Verus crate pattern. The `wf()` invariant enforces correctness for API-reachable values.
5. The verification proves 97 obligations including: default emptiness, set/clear correctness, bit preservation, idempotency, roundtrip properties, well-formedness preservation, mask equivalence, enum closure, and set-level bridging lemmas.
