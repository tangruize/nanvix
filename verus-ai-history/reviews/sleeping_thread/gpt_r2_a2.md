# Review: sleeping_thread (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `join_cond()` omission in `verus/split/kernel/pm/thread/sleeping.rs` (exec)
  - **Description:** `join_cond()` is still not modeled or verified. The updated comments justify the omission but do not provide a proof model or spec, so a public API remains outside the verification boundary.
  - **Suggested Fix:** Add an opaque Condvar token (or a boundary model) and verify `join_cond()` as a pass-through, or explicitly state and enforce a module-level invariant tying condvar identity to ThreadState.

- **Location:** `thread_state_mut()` in `verus/split/kernel/pm/thread/sleeping.rs` (exec)
  - **Description:** Still marked `#[verifier::external]` with no enforced postconditions. The added documentation does not prevent callers from violating `wf`, `spec_id`, or `spec_alarm`; this remains a trust hole.
  - **Suggested Fix:** Replace with verified setters/borrowed-logic wrappers or add a verified `&mut` API when Verus supports it.

### Medium
- **Location:** `spec_valid_reason` in `verus/split/kernel/pm/thread/sleeping.spec.rs`
  - **Description:** The mapping of `InterruptReason` to int tags is still TODO and unproven, so the enum↔int correspondence remains a semantic gap.
  - **Suggested Fix:** Add a verified conversion lemma in the InterruptReason module and discharge the TODO.

- **Location:** `ReadyThread`/`InterruptedThread` boundary models in `verus/split/kernel/pm/thread/sleeping.rs`
  - **Description:** Cross-module correctness is still assumed. The new `clock_now()` boundary only guarantees `>= 0` and there is still no proof that real `ready.rs`/`interrupted.rs` specs imply these postconditions.
  - **Suggested Fix:** Verify the sibling modules and prove these boundary postconditions; if scheduling logic needs stronger time properties, strengthen `clock_now()` accordingly.

### Low
- None.

## Positive Observations
- Trust boundaries are now clearly documented, including cross-module obligations and the clock boundary.
- `thread_state()`/`id()` specs and boundary `clock_now()` wiring improve spec clarity.

## Summary
The updates add documentation and a `clock_now()` boundary, but the key gaps from the prior review remain: `join_cond()` is still unmodeled, `thread_state_mut()` is still an external trust hole, and the enum/int mapping and cross-module boundary obligations are still unproven. Verification is therefore not complete or fully sound.
