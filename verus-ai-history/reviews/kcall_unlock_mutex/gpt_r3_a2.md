# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `unlock_mutex_model` preconditions (exec: `verus/split/kernel/pm/kcall/unlock_mutex.rs`).
  **Description:** The model still requires `spec_is_currently_running(pid, tid)` even though the original function does not enforce this and `pid/tid` do not affect behavior beyond logging. No dispatcher proof is referenced here, so equivalence remains contingent on an external invariant that is not linked.
  **Suggested Fix:** Either discharge this precondition in the verified kcall dispatch layer and reference it here, or remove it and stop relying on `pid/tid` to justify `spec_thread_owns_mutex` in this module.

- **Location:** `take_mutex_guard_model` postconditions (exec: `verus/split/kernel/pm/kcall/unlock_mutex.rs`).
  **Description:** The error path still does not guarantee that when `pm_internally_dropped_guard == false` the mutex remains locked/unchanged. The model remains weaker than the PM implementation and could hide regressions where error paths incorrectly unlock.
  **Suggested Fix:** Add a postcondition tying `pm_internally_dropped_guard == false` on error to a predicate that the mutex state is unchanged (or at least not unlocked), and prove it from the PM model.

### Low
- **Location:** `spec_is_valid_error_code` (spec: `verus/split/kernel/pm/kcall/unlock_mutex.spec.rs`).
  **Description:** The predicate still accepts any positive integer, not just actual `ErrorCode` values. This weakens error-code validity.
  **Suggested Fix:** Replace with a predicate that matches the `ErrorCode` enum (or reuse an existing spec from the error crate).

- **Location:** Liveness/notification effects (spec/proof: `unlock_mutex.spec.rs`, `unlock_mutex.proof.rs`).
  **Description:** The model continues to omit the “threads notified on unlock” liveness effect mentioned in the original code comments.
  **Suggested Fix:** If liveness is intended at this boundary, add an abstract notification predicate to `drop_guard_model` or explicitly cite a proved property from the mutex module.

## Positive Observations
- The exec model still matches the original control flow and returns the correct success/error mapping.
- The guard-drop modeling and token chain remain clear and well-separated from the spec/proof layers.
- Trust boundaries are documented explicitly with rationale for PM-internal error paths.

## Summary
The previous issues were not actually fixed in the updated files; the same strengthened `pid/tid` precondition and weak error-path mutex-state guarantees remain. Error-code validity and liveness modeling are also still weak or out of scope. Verification is solid structurally but not yet complete for the intended behavioral equivalence.
