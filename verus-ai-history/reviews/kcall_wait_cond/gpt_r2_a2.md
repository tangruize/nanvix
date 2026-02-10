# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `wait_cond.rs` (exec, module docs)
  **Description:** Liveness is still explicitly out of scope; there is no proof or assumption that `cond.wait` or `mutex.lock(None)` eventually returns (under fairness/timeout assumptions). For a wait primitive, eventual return is a key property.
  **Suggested Fix:** Add a liveness lemma (or reference an existing scheduler/condvar liveness proof) and thread it as an explicit assumption in this module’s contract.

### Low
- None.

## Positive Observations
- The explicit `USIZE_BITS() == 32` precondition now makes the 32-bit architectural assumption explicit at the model boundary.
- Resource-release invariants were strengthened: `spec_mutex_released`, `spec_cond_ref_released`, and `spec_mutex_reacquired` are now asserted whenever the corresponding steps succeed, even if later steps fail.
- Exec-spec equivalence and stored-result semantics remain faithfully modeled, with clean separation of exec/spec/proof and clear trust boundaries.

## Summary
Previous issues about architectural assumptions and resource-release invariants were addressed with explicit preconditions and strengthened postconditions. The only remaining gap is liveness, which is still deferred to other modules without an explicit assumption here.
