# Review: process_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `Capabilities` struct visibility and `wf` invariant (exec: `verus/split/kernel/pm/process/capability.rs`, spec: `verus/split/kernel/pm/process/capability.spec.rs`).
  **Description:** The previous precondition on `set`/`clear` was removed (good), but the `bits`
  field remains `pub`, so external code can construct non-`wf` states that are impossible in the
  original module (private tuple field). The documentation asserts a module-level invariant, but it
  is not enforced, so API equivalence still does not hold and downstream proofs may silently assume
  `wf()` on values that can be invalidated by direct field writes.
  **Suggested Fix:** Hide the field (e.g., private tuple struct) and keep spec access via `View` or
  make `spec_bits()`/`spec_has()` non-public (`pub(crate)`), or introduce an actual invariant
  (`invariant self.wf()`) and eliminate public mutation of `bits` so the invariant is enforced.

### Low
- None.

## Positive Observations
- The earlier `set`/`clear` preconditions were removed and replaced with conditional `wf()`
  preservation, so functional semantics are now total and match the original bitwise behavior.
- The closed-world assumption is now explicitly proven (`lemma_enum_is_closed`) and documented,
  making the mapping choice auditable and less error-prone on future changes.
- Verification still avoids `assume`/`external_body` and proves key safety properties (idempotence,
  roundtrips, and preservation of other bits).

## Summary
The rework fixes the precondition weakness and makes the closed-world assumption explicit, but the
public `bits` field still changes the public API and leaves the stated invariant unenforced.
Tightening encapsulation or enforcing the invariant is needed to fully restore equivalence.
