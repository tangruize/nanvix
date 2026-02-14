# Exec Consistency Fix: process_capability

## Summary
- Mismatches fixed: 0 (all 3 are documented equivalences — no logic changes)
- Missing functions added: 0
- Documented equivalences: 6 (3 mismatches + 3 extras)

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `Capabilities` (struct) | Documented equivalence | Original: `Capabilities(u8)` tuple struct. Verus: `Capabilities { pub bits: u8 }` named struct. Verus requires `pub` fields for `pub open spec fn` definitions that reference struct fields. `pub(crate)` and `pub(super)` both fail with Verus error "must be well-formed everywhere". This is the established pattern in the Nanvix verification crate (cf. `ProcessIdentifier.value`). The field `bits` is functionally identical to the tuple field `.0`. |
| `set` | Documented equivalence | Original: `self.0 \|= 1 << capability as u8`. Verus: `let mask = Self::to_mask(capability); self.bits = self.bits \| mask`. The `to_mask()` function returns the same values as `1 << discriminant` for all 5 variants (proven by `lemma_mask_matches_discriminant`). Verus does not support the `<<` bit-shift operator on enum discriminant casts in exec mode, requiring the explicit match. The OR-assign logic is identical. |
| `clear` | Documented equivalence | Original: `self.0 &= !(1 << capability as u8)`. Verus: `let mask = Self::to_mask(capability); self.bits = self.bits & !mask`. Same justification as `set` — `to_mask()` replaces the shift expression, AND-NOT logic is identical. |
| `has` | Documented equivalence | Original: `(self.0 & (1 << capability as u8)) != 0`. Verus: `let mask = Self::to_mask(capability); (self.bits & mask) != 0u8`. Same justification — `to_mask()` replaces the shift expression, comparison logic is identical. |
| `to_mask` | Justified extra | Helper function extracted for verification. Replaces `1 << capability as u8` with explicit match because Verus cannot compile bit-shift on enum discriminant casts. Returns identical values: ExceptionControl→1, InterruptControl→2, IoManagement→4, MemoryManagement→8, ProcessManagement→16. Equivalence proven by `lemma_mask_matches_discriminant` which shows `spec_mask(cap) == spec_pow2_mask(cap.spec_discriminant())` for all variants. |
| `new` | Justified extra | Constructor providing verified postconditions (`wf()` and empty granted set). The original source has no explicit `new()` — it relies on `#[derive(Default)]`. This constructor is needed for Verus because derived traits cannot carry `ensures` clauses. Produces `Capabilities { bits: 0u8 }`, identical to the derived `Default`. |
| `default` | Justified extra | Explicit `Default` trait impl replacing `#[derive(Default)]`. Verus cannot derive `Default` with postconditions. The implementation returns `Capabilities { bits: 0u8 }`, identical to what `#[derive(Default)]` would produce for the original `Capabilities(u8)`. |

## Equivalence Argument

### Mask Computation Equivalence
The original source computes capability masks via `1 << capability as u8`, which relies on
Rust's implicit enum discriminant assignment (sequential from 0). The `Capability` enum has
no `#[repr]` attribute, and discriminants are confirmed as 0..=4 by the `TryFrom<u32>` impl
in `sys::pm::capability`. The Verus `to_mask()` function returns `2^d` for each discriminant
`d`, matching `1 << d` exactly:

| Variant | Discriminant | `1 << d` | `to_mask()` | Match |
|---------|-------------|----------|-------------|-------|
| ExceptionControl | 0 | 1 | 1 | ✅ |
| InterruptControl | 1 | 2 | 2 | ✅ |
| IoManagement | 2 | 4 | 4 | ✅ |
| MemoryManagement | 3 | 8 | 8 | ✅ |
| ProcessManagement | 4 | 16 | 16 | ✅ |

This equivalence is formally proven by `lemma_mask_matches_discriminant` in the proof file.

### Struct Representation Equivalence
`Capabilities(u8)` and `Capabilities { bits: u8 }` are isomorphic single-field wrappers
around `u8`. All field accesses `.0` map to `.bits`. The `pub` visibility of `bits` is wider
than the original private tuple field, but the verification's `wf()` invariant ensures
correctness for API-reachable values (documented in module header).

## Verification: PASS
- Command: `verus --crate-type lib lib.rs --verify-module kernel::pm::process::capability`
- Result: 98 verified, 0 errors
- No `assume`, `admit`, or unjustified `external_body` found
