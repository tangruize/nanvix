# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `unlock_mutex_model` preconditions (exec: `verus/split/kernel/pm/kcall/unlock_mutex.rs`).
  **Description:** The strengthened precondition `spec_is_currently_running(pid, tid)` is still present without a linked proof from the kcall dispatch layer. The original code does not enforce this, and `pid/tid` only affect logging, so equivalence remains contingent on an external invariant not shown here.
  **Suggested Fix:** Discharge this precondition in a verified dispatcher module and reference it, or remove it and avoid relying on `pid/tid` to establish `spec_thread_owns_mutex` in this module.

- **Location:** `take_mutex_guard_model` postconditions (exec: `verus/split/kernel/pm/kcall/unlock_mutex.rs`).
  **Description:** On error with `pm_internally_dropped_guard == false`, the model still leaves the mutex state unconstrained. The concrete PM implementation implies no unlock on this path, so the spec remains too weak and could mask regressions.
  **Suggested Fix:** Add a postcondition that `pm_internally_dropped_guard == false` implies the mutex is not unlocked (or state unchanged), and tie it to PM proofs.

### Low
- **Location:** `spec_is_valid_error_code` (spec: `verus/split/kernel/pm/kcall/unlock_mutex.spec.rs`).
  **Description:** Still accepts any positive integer rather than restricting to `ErrorCode` values.
  **Suggested Fix:** Replace with an enum-matching predicate (or reuse an error-crate spec predicate).

- **Location:** Liveness/notification effects (spec/proof: `unlock_mutex.spec.rs`, `unlock_mutex.proof.rs`).
  **Description:** The “threads notified on unlock” liveness effect remains out of scope with no abstraction or cross-module link.
  **Suggested Fix:** Add an abstract notification predicate or explicitly cite a proved mutex-module property.

## Positive Observations
- Exec, spec, and proof separation remains clean and aligned with control flow.
- Guard-drop modeling and error propagation properties are still clearly proved.
- Trust-boundary documentation is thorough and internally consistent.

## Summary
No substantive changes were observed in the updated files, and the previously reported issues remain. Verification is structurally solid but still not complete for behavioral equivalence due to the strengthened pid/tid precondition and weak error-path mutex-state guarantees.
