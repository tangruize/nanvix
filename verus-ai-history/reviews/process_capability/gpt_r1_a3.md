# Review: process_capability (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `Capabilities` struct visibility and invariant enforcement (exec: `verus/split/kernel/pm/process/capability.rs`, spec: `verus/split/kernel/pm/process/capability.spec.rs`).
  **Description:** The prover added documentation and an `lemma_api_preserves_wf`, but the field is
  still `pub` and there is still no enforced type invariant. This means external code can construct
  or mutate `Capabilities { bits: 0xE0 }` and observe behavior that is impossible in the original
  module (private tuple field). The proof only covers API-reachable states; equivalence to the
  original module’s encapsulation remains unproven.
  **Suggested Fix:** Enforce encapsulation (private field or tuple struct) and keep spec access via
  `View`/trusted accessors, or introduce a true invariant (`invariant self.wf()`) with no public
  mutation of `bits`. If Verus visibility constraints force `pub`, document and prove a wrapper API
  that is the only exported type, or gate raw access behind a `cfg(verus)` boundary.

### Low
- None.

## Positive Observations
- The closed-world assumption is now explicitly proven (`lemma_enum_is_closed`) and the API
  preservation lemma clarifies the intended invariant on API-reachable states.
- `set`/`clear` are total and preserve semantics without preconditions, matching the original bitwise
  operations.
- No `assume`/`external_body` is used; key safety properties remain proven.

## Summary
The update improves documentation and formalizes the API-reachable invariant, but it does not fix the
core equivalence gap introduced by the `pub` bits field. Encapsulation or a true enforced invariant is
still required for full soundness relative to the original module.
