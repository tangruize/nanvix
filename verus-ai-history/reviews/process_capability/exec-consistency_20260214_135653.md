# Review: process_capability Exec Consistency (claude-opus-4.6)

## Grade: A

## Summary

The exec consistency fixes are well-executed. All three original functions (`set`, `clear`, `has`) are faithfully reproduced with documented, formally proven equivalences. The struct representation change and `to_mask` helper are necessary Verus adaptations with sound justifications. Verification passes cleanly (98 verified, 0 errors) with no `assume`, `admit`, or `external_body`. One minor concern (widened field visibility) is properly documented and mitigated by the `wf()` invariant pattern. Grade is A rather than A+ due to the inherent visibility widening risk and the density of verification-only additions relative to the 3-function original.

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes — all 3 mismatches documented as equivalences.**

The consistency report identifies 3 mismatched functions (`set`, `clear`, `has`). None required code restoration; all are documented equivalences where `to_mask(capability)` replaces `1 << capability as u8`. This is correct:

- **`set`**: Original `self.0 |= 1 << capability as u8` → Verus `self.bits = self.bits | Self::to_mask(capability)`. Semantically identical; OR-assign logic preserved.
- **`clear`**: Original `self.0 &= !(1 << capability as u8)` → Verus `self.bits = self.bits & !Self::to_mask(capability)`. AND-NOT logic preserved.
- **`has`**: Original `(self.0 & (1 << capability as u8)) != 0` → Verus `(self.bits & Self::to_mask(capability)) != 0u8`. Comparison logic preserved.

The mask table in the report correctly maps discriminants 0–4 to values 1, 2, 4, 8, 16, matching `1 << d` for each.

### 2. Were MISSING functions added with proper verification?

**No missing functions were identified — correct.**

The original source has exactly 3 methods (`set`, `clear`, `has`) plus a `#[derive(Default)]`. All are present in the Verus version. The `Default` derive is replaced by an explicit `Default` impl (justified extra). No functions were omitted.

### 3. Are equivalence justifications sound?

**Yes — all justifications are sound and formally verified.**

| Equivalence | Soundness | Verification |
|---|---|---|
| `to_mask` ↔ `1 << discriminant` | Correct. The 5 match arms return the same values as bit-shifting by discriminants 0–4. No `#[repr]` is needed because the mapping is explicit in both the original `TryFrom<u32>` and the Verus match. | Proven by `lemma_mask_matches_discriminant` via exhaustive case analysis. |
| Tuple struct ↔ named struct | Correct. `Capabilities(u8)` and `Capabilities { bits: u8 }` are isomorphic single-field wrappers. | N/A (structural equivalence). |
| `#[derive(Default)]` ↔ explicit `Default` impl | Correct. Both produce `{ bits: 0u8 }` / `(0u8)`. | `new()` and `default()` both ensure `wf()` and empty granted set. |

### 4. Does the exec code faithfully represent the original source?

**Yes, with documented deviations.**

Function-by-function comparison:

| Original | Verus exec | Faithful? |
|---|---|---|
| `Capabilities(u8)` tuple struct | `Capabilities { pub bits: u8 }` named struct | Equivalent (documented) |
| `#[derive(Default, Clone, Copy)]` | `#[derive(Clone, Copy, PartialEq, Eq)]` + explicit `Default` | Equivalent + additions for Verus |
| `set(&mut self, capability)` | Same signature, same body logic with `to_mask` | ✅ |
| `clear(&mut self, capability)` | Same signature, same body logic with `to_mask` | ✅ |
| `has(&self, capability) -> bool` | Same signature, same body logic with `to_mask` | ✅ |

Additions not in original (all justified):
- `new()` constructor — needed because `#[derive(Default)]` can't carry Verus postconditions
- `to_mask()` helper — needed because Verus can't compile `1 << enum_variant as u8`
- `PartialEq`/`Eq` derives — needed for Verus equality reasoning
- `CapabilitiesView` + `View` impl — Verus abstraction pattern
- All spec/proof functions — verification infrastructure

### 5. Does verification still pass?

**Yes.** `98 verified, 0 errors`. No `assume`, `admit`, or `external_body` found in any of the three files.

## Issues Found

### Critical

None.

### Major

None.

### Minor

1. **Widened field visibility (`pub bits`)**: The original uses a private tuple field; the Verus version exposes `pub bits`. This permits construction of non-`wf()` values that bypass the `set`/`clear` API. The risk is thoroughly documented (module header lines 46–82) and mitigated by the `wf()` invariant pattern with `lemma_api_preserves_wf`. However, this is a genuine deviation from the original's encapsulation. The documentation correctly acknowledges this as an "unavoidable consequence of Verus's visibility rules" and notes that alternatives (`pub(crate)`, `pub(super)`, private) were tested and rejected. **Acceptable given Verus tooling constraints.**

2. **Added `PartialEq`/`Eq` derives not in original**: The original derives `Default, Clone, Copy`. The Verus version adds `PartialEq, Eq`. These are semantically safe (auto-derived equality on a `u8` wrapper is well-defined) and needed for Verus equality reasoning. **No behavioral impact.**

### Observations

- The proof file is comprehensive with 20+ lemmas covering idempotency, roundtrip, well-formedness preservation, mask disjointness, set-level bridging, and closed-world enumeration.
- The two-level spec abstraction (bit-level + set-level) with bridging lemmas is a good design that avoids the "spec mirrors implementation" anti-pattern.
- The `spec_default` is correctly `closed` to hide representation, with properties exposed via lemmas.
- The `wf()` predicate is correctly `closed` per the methodology.
- Documentation quality is excellent — module header explains every deviation, every design decision, and the trust boundary.
