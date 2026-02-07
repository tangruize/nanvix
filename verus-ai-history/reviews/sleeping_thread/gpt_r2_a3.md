# Review: sleeping_thread (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `join_cond()` omission in `verus/split/kernel/pm/thread/sleeping.rs` (exec)
  - **Description:** `join_cond()` remains unmodeled; the code still omits the API and only
    documents the decision. This leaves a public method outside the proof boundary.
  - **Suggested Fix:** Add an opaque Condvar token or boundary model and verify `join_cond()` as a
    pass-through, or enforce a module-level invariant tying condvar identity to ThreadState.

- **Location:** `thread_state_mut()` in `verus/split/kernel/pm/thread/sleeping.rs` (exec)
  - **Description:** Still `#[verifier::external]` with no enforced postconditions. Documentation
    does not prevent callers from violating `wf`, `spec_id`, or `spec_alarm`.
  - **Suggested Fix:** Replace with verified setters/borrowed-logic wrappers or add a verified
    `&mut` API when Verus supports it.

### Medium
- **Location:** `spec_valid_reason` in `verus/split/kernel/pm/thread/sleeping.spec.rs`
  - **Description:** The enum↔int mapping remains TODO and unproven; soundness still depends on
    external verification of `InterruptReason`.
  - **Suggested Fix:** Add a verified conversion lemma in the InterruptReason module and discharge
    the TODOs.

- **Location:** `ReadyThread`/`InterruptedThread` boundary models in
  `verus/split/kernel/pm/thread/sleeping.rs`
  - **Description:** Cross-module correctness is still assumed; there is no evidence the real
    sibling module specs imply these postconditions. `clock_now()` still only ensures `>= 0`.
  - **Suggested Fix:** Verify the sibling modules and prove the boundary postconditions; if
    scheduling logic needs stronger time properties, strengthen the time model accordingly.

### Low
- None.

## Positive Observations
- Trust boundaries and cross-module obligations remain clearly documented.

## Summary
No substantive fixes were made to the prior issues; the key verification gaps remain. The module
still relies on unmodeled public APIs and unproven cross-module assumptions, so verification is not
complete or fully sound.
