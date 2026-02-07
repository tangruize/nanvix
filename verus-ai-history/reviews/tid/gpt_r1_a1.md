# Review: tid (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Priority:** Medium  
  **Location:** `ThreadIdentifier` struct (exec, `verus/split/kernel/pm/sys/tid.rs`)  
  **Description:** The verified exec type exposes the `value` field as `pub`, whereas the original tuple struct keeps its field private. This weakens encapsulation and changes the public API, allowing direct construction or mutation of the raw value without using the conversion helpers.  
  **Suggested Fix:** Make the field private and add `#[verifier::opaque]` or accessor methods for spec reasoning (e.g., keep `spec_value` via `view()`), or wrap the public field behind a private module-level constructor used only in proofs.

- **Priority:** Medium  
  **Location:** `impl Default for ThreadIdentifier` (exec, `verus/split/kernel/pm/sys/tid.rs`)  
  **Description:** The verified code introduces a `Default` implementation that is not present in the original source. This adds an API surface and behavior (implicit default construction) not available in the production code, which violates strict equivalence.  
  **Suggested Fix:** Remove the `Default` impl from the verified exec module, or add the same `Default` impl to the original Rust source if it is intended.

### Low
- **Priority:** Low  
  **Location:** `eq`/`ne`/`lt`/`le`/`gt`/`ge`/`cmp_ord`/`default_value` methods (exec, `verus/split/kernel/pm/sys/tid.rs`)  
  **Description:** The verified module introduces extra inherent methods that are not present in the original type. While harmless for proof convenience, they expand the public API relative to the original.  
  **Suggested Fix:** Restrict these methods to a private module or gate them behind `#[verifier::external]`/`pub(crate)` so the verified API matches the original surface.

## Positive Observations
- All original conversions and byte-serialization functions have verified counterparts with precise success/error conditions.
- Error paths explicitly enforce `ErrorCode::InvalidArgument` with the correct message, matching the original behavior.
- Layout and endian round-trip properties are documented and isolated as explicit trust boundaries.
- Ordering semantics are fully specified and proved consistent with `Ordering`.

## Summary
The verification is strong and largely equivalent to the original behavior, with clear specs and well-scoped axioms for byte serialization and layout. The main gaps are API-surface mismatches (public field exposure and an extra `Default` impl), which should be corrected for strict semantic equivalence. After aligning the public interface, the proof quality would be solid for this component.
