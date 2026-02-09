# Review: process_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Priority:** Medium  
  **Location:** `Capabilities` struct (`capability.rs`, exec) and `wf()` invariant (`capability.spec.rs`, spec)  
  **Description:** The previous encapsulation issue remains unresolved: `Capabilities` still exposes `pub bits`, which makes the verified executable interface more permissive than the original private tuple field. This allows external construction/mutation of non-`wf` states while the proofs only guarantee correctness for API-reachable states. The file still explicitly documents the deviation, confirming it was not fixed in code.  
  **Suggested Fix:** Preserve the private storage boundary (e.g., wrap the raw field in a private inner struct with `pub` bits only for spec access, or seal construction in the module and expose only constructors/mutators). If visibility constraints remain, add a verified wrapper type and restrict public construction to restore semantic equivalence.

### Low
- None.

## Positive Observations
- Coverage remains complete for `set`, `clear`, and `has`, with postconditions matching bitwise behavior.
- The mask/discriminant equivalence proof is intact and still avoids reliance on enum layout.
- Verification still uses no `assume`/`external_body`, and proof/spec separation is clean.

## Summary
The re-review finds no substantive changes to address the prior medium issue: encapsulation is still weaker than the original source due to `pub bits`. No new issues were introduced and verification still passes, but the semantic mismatch remains and should be fixed to claim full equivalence.
