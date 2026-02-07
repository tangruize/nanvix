# Review: tid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** Trait implementations in `tid.rs` (exec, outside `verus!`).
  **Description:** The public trait methods (`From`, `TryFrom`, `PartialEq`, `Ord`, `Debug`) that correspond to the original public API live outside the `verus!` block and are therefore not verified. The verified helpers (`from_i32`, `try_from_isize`, `cmp_ord`, etc.) exist, but the wrappers that callers actually invoke are trusted glue, which weakens the “all functions verified” coverage claim.
  **Suggested Fix:** Move the trait implementations into a `verus!` block with proper postconditions if supported, or mark them as `#[verifier::external_body]` with explicit ensures and document them as trusted boundaries. Alternatively, expose only the verified helper APIs and restrict trait impls to non-verified build targets.

- **Location:** `spec_to_ne_bytes` / `spec_from_ne_bytes` in `tid.spec.rs` and byte axioms in `tid.proof.rs`.
  **Description:** The specification for byte serialization only enforces round-trip properties via axioms, but it does not characterize the actual native-endian layout of `i32`. This is too weak to prove ABI/serialization compatibility beyond round-trip fidelity.
  **Suggested Fix:** Strengthen the spec to relate bytes to `value` explicitly (e.g., via a bitvector model or an axiom that matches `i32::to_ne_bytes` semantics), and use that in proofs where serialization format matters.

### Low
- **Location:** `assert_layout` in `tid.proof.rs`.
  **Description:** The layout proof lemma is never invoked, so there is no module-level proof obligation mirroring the original `static_assert` usage. The evidence exists but is not asserted as part of the verified module.
  **Suggested Fix:** Add a small proof entry point (or a proof block in the exec module) that calls `ThreadIdentifier::assert_layout()` to assert ABI layout properties in this module’s proof surface.

## Positive Observations
- Coverage of conversion logic is strong: each conversion has explicit pre/postconditions, and error codes/reasons match the original behavior.
- Ordering, equality, and constant-value properties are explicitly specified with supporting proof lemmas.
- Spec/proof separation is clean, and trust boundaries for layout and byte serialization are clearly documented.

## Summary
The verification captures the main functional behavior and error semantics of `ThreadIdentifier`, but some public API wrappers remain unverified and the byte-serialization spec is weaker than the concrete ABI requirements. Strengthening the trait-impl verification boundary and tightening the serialization specification would elevate the assurance level.
