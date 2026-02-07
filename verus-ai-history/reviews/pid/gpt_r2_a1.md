# Review: pid (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessIdentifier::assert_layout` (proof, `pid.proof.rs`)  
  **Description:** The layout proof (`assert_layout`) is never invoked, so the size/alignment properties that mirror the original `static_assert` are not actually checked by verification. This leaves the ABI/layout requirement effectively unenforced in the verified module.  
  **Suggested Fix:** Invoke `ProcessIdentifier::assert_layout()` from a top-level proof function or a module-level proof harness so the lemmas are exercised (or use a `const _: () = { ... }` style proof call if supported by Verus).

- **Location:** Trait impls outside `verus!` (exec, `pid.rs`)  
  **Description:** The `Default`, `PartialEq`, `Ord`, `Debug`, `From`, and `TryFrom` trait implementations are not verified (they live outside `verus!`). This means several original functions are only assumed to behave correctly via wrappers, which is a coverage gap against the stated criteria.  
  **Suggested Fix:** Move these impls into a verified `verus!` context with specs (or provide verified wrapper methods that fully characterize them), and mark only the minimal glue as `external_body` if necessary.

### Low
- **Location:** `ProcessIdentifier` struct field visibility (exec, `pid.rs`)  
  **Description:** The verified struct exposes `pub value: i32`, whereas the original type is a private tuple field. This relaxes encapsulation and could let other verified modules rely on direct field access that is not available in the real code, weakening equivalence.  
  **Suggested Fix:** Keep the field private and expose a `pub` spec getter (or `pub(crate)` with restricted access) so proofs do not rely on a public field that does not exist in the original API.

- **Location:** Byte/layout axioms (`pid.rs`, `pid.proof.rs`)  
  **Description:** The byte round-trip and layout facts are modeled with `external_body` axioms. They are documented, but still trusted assumptions inside a core kernel module.  
  **Suggested Fix:** Isolate these axioms in a clearly designated trusted module or replace them with Verus library proofs if available, to reduce the trusted base.

## Positive Observations
- Verified exec functions cover all conversion paths with explicit range checks and precise error-code/reason guarantees, matching the original `try_into`/`try_from` behavior.
- Specs cleanly capture the intended semantics for value preservation, ordering relations, and error handling.
- The split between exec/spec/proof is clear and consistent, with good documentation of trust boundaries for byte serialization and layout.

## Summary
The verification is solid for value conversions and error handling, and the spec/proof split is well-structured. The main gaps are unexercised layout checks and unverified trait impls, plus minor equivalence risks from a public field and trusted axioms. Addressing these would raise confidence that the verified module fully matches the original kernel component.
