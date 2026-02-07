# Review: tid (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ThreadIdentifier` struct field visibility (exec, `verus/split/kernel/pm/sys/tid.rs`).
  **Description:** The verified exec type exposes `pub value: i32`, while the original type is a tuple struct with a private field. This changes the public API and allows external code to read/mutate the raw value directly, bypassing the intended constructor/conversion paths and weakening semantic equivalence. 
  **Suggested Fix:** Make the field private (or `pub(crate)`) to match the original API and rely on `spec_value()`/`view()` for reasoning. If Verus needs access, use accessor methods or a `view()` spec rather than a public field.

### Low
- **Location:** Byte serialization and layout lemmas (proof, `tid.proof.rs`; exec, `tid.rs`).
  **Description:** `to_ne_bytes`, `from_ne_bytes`, and layout checks rely on `#[verifier::external_body]` axioms. These are documented but remain trusted assumptions inside a core module, so the soundness of byte round-trips and ABI layout is not proved. 
  **Suggested Fix:** Isolate these assumptions in a designated trusted/axiom module, or replace with a proven model (e.g., bitvector-based spec) if available in the Verus standard library.

## Positive Observations
- Coverage is complete: all original conversions, byte helpers, and trait behaviors are represented in exec/spec/proof with explicit specs.
- Error semantics are preserved consistently (`InvalidArgument` with the same message) and range checks match the original `try_into` behavior.
- Split quality is strong: specs live in `tid.spec.rs`, proofs in `tid.proof.rs`, exec code is clean and uses verified helpers.

## Summary
Verification is thorough for conversions, ordering, and constants, with clear specs and proofs. The main gaps are minor: public field exposure slightly breaks API equivalence and trusted byte/layout axioms remain assumptions. Tightening encapsulation and isolating/justifying axioms would push this to an A/A+.
