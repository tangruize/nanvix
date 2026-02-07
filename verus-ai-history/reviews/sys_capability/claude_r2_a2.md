# Review: sys_capability (Round 2, Attempt 2)

## Grade: A+

## Previous Issue Resolution

### Medium-1: `spec_from_discriminant` default case (FIXED ✅)

**Previous concern:** Out-of-range inputs silently returned `Capability::ProcessManagement`, risking masked logic errors if the spec function was used outside guarded contexts.

**Verification:** The else branch on line 68 of the spec file now returns `arbitrary()` instead of a specific variant. This is the correct fix — it makes the out-of-range behavior explicitly unspecified, preventing any downstream proof from accidentally relying on a concrete default. The `recommends` clause is retained as a usage hint. Confirmed fixed.

### Low-1: `to_u32` documentation (FIXED ✅)

**Previous concern:** `to_u32` is not present in the original source and should be documented as a verification addition.

**Verification:** The exec file header (lines 17–24) now includes a "Verification Additions" section explicitly listing `to_u32` as an auxiliary function. The function's own doc comment (lines 119–122) also states it is not present in the original source. Confirmed fixed.

### Low-2: `CapabilityView` underutilized (ADDRESSED ✅)

**Previous concern:** The view abstraction was defined but rarely used in proofs.

**Verification:** The exec file header (lines 22–23) now documents `CapabilityView` as added "for composability with larger verified modules." The prover chose to retain the abstraction as forward-looking infrastructure rather than remove it or artificially force its use. This is a reasonable design decision — the view provides a clean integration point for future `Capabilities` bitfield verification where set-level reasoning would benefit from view-based abstraction. `lemma_view_equality` already demonstrates the view's utility for discriminant-based equality reasoning. Accepted.

### Low-3: `PartialEq`/`Eq` derives (FIXED ✅)

**Previous concern:** Original source only derives `Debug, Clone, Copy`; adding `PartialEq, Eq` expands the trait surface.

**Verification:** The exec file header (line 21) now explicitly documents these derives as "required by Verus for equality reasoning." This is accurate — Verus needs structural equality for `*a == *b` assertions in proof lemmas like `lemma_discriminants_disjoint` and `lemma_view_equality`. Confirmed fixed.

### Low-4: Missing `#[repr]` assumption (FIXED ✅)

**Previous concern:** Discriminant layout assumptions were not documented.

**Verification:** The Trust Boundary section (lines 29–31) now states: "Discriminant values (0..=4) are explicitly mapped in both exec and spec code; no `#[repr]` annotation is used, so the mapping does not depend on compiler-assigned layout." This correctly identifies that the explicit match arms in both `try_from_u32`/`to_u32` (exec) and `spec_discriminant`/`spec_from_discriminant` (spec) constitute a manual mapping that does not rely on `#[repr]`. Confirmed fixed.

## New Issues Found

None.

## Verification Integrity Check

- **No `assume`, `admit`, `external_body`, or `trusted` annotations** anywhere in the three module files. Confirmed via grep.
- **Original source faithfulness:** The original `Capability` enum (5 variants, same names, same order) and `TryFrom<u32>` implementation (same match arms, same error code and message) are faithfully reproduced. The only additions are documented verification auxiliaries.
- **Proof completeness:** All 9 proof lemmas have empty bodies, meaning Verus's SMT solver verifies them automatically from the spec definitions. This is the strongest form of proof — no manual assertions, hints, or workarounds needed.
- **Spec soundness:** `spec_from_discriminant` with `arbitrary()` for out-of-range, `spec_discriminant` exhaustively covering all variants, `spec_is_valid_discriminant` matching the range [0, 4] — all consistent and complete.

## Positive Observations

- **All previous issues genuinely resolved.** Every fix was substantive and correct, not superficial. The `arbitrary()` change is the most important — it eliminates the only real semantic concern from the prior review.

- **Excellent documentation.** The exec file header now serves as a clear contract: verified properties, verification additions, and trust boundary are all explicitly stated. A reader unfamiliar with the module can immediately understand what is verified, what was added, and what assumptions exist.

- **Clean separation maintained.** The three-file structure (exec/spec/proof) remains well-organized with no cross-contamination of concerns.

- **Faithful to original source.** Comparing against `src/libs/sys/src/sys/pm/capability.rs`, the verified code preserves all original semantics while adding only the documented auxiliary infrastructure.

## Summary

All five issues from the previous review have been genuinely fixed. The medium-priority `spec_from_discriminant` default case now correctly uses `arbitrary()` for out-of-range inputs. All low-priority documentation gaps have been filled with clear, accurate descriptions in the exec file header. No new issues were introduced. The verification is complete, sound, and well-documented with zero trusted assumptions.
