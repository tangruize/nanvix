# Review: sleeping_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `ReadyThread` boundary model in `sleeping.rs` (exec), lines 123-129 vs. `interrupted.rs` boundary model (lines 92-95)
- **Description:** The `ReadyThread` boundary model in `sleeping.rs` includes an `admission_time: int` field, while the `ReadyThread` boundary model in the sibling `interrupted.rs` module omits it entirely. Both are boundary models of the same canonical `ReadyThread` type. Although both are individually sound (each documents its scope), the structural divergence means cross-module lemmas cannot directly compose: a `ReadyThread` produced by `SleepingThread::wakeup()` has type `sleeping::ReadyThread{state, admission_time}`, while one produced by `InterruptedThread::resume()` has type `interrupted::ReadyThread{state}`. If a future cross-module proof needs to reason about ReadyThread values from different origins, these incompatible types will require reconciliation. This is an architectural debt in the boundary model strategy.
- **Suggested Fix:** Standardize boundary models across modules. Either all boundary `ReadyThread` models include `admission_time`, or a shared canonical boundary type is extracted into a common module.

### Medium

- **Location:** `thread_state_mut()` in `sleeping.rs` (exec), lines 418-421
- **Description:** `thread_state_mut()` is `#[verifier::external]` and returns `&mut ThreadState`. This is the only function where a caller can silently violate the `wf()`, `spec_id()`, and `spec_alarm()` invariants without any machine-checked postconditions. The trust boundary documentation is thorough (lines 394-417), and the suggestion to use `set_thread_data_area()` as a verified alternative is good. However, any caller using `thread_state_mut()` operates entirely outside the verification boundary, which is a meaningful gap in an OS kernel context.
- **Suggested Fix:** Audit all callers of `thread_state_mut()` to verify they only perform operations already covered by verified setter methods (e.g., `set_thread_data_area`). Where possible, add specific verified setter methods for remaining mutation patterns, reducing reliance on `thread_state_mut()`. Document which callers exist and what mutations they perform.

- **Location:** `clock_now()` boundary function in `sleeping.rs` (exec), lines 79-85
- **Description:** `clock_now()` is `external_body` with postcondition `result >= 0`. This is used in the `ReadyThread::from_state` boundary model. The same function appears separately in `ready.rs` (line 74-80) with the same signature, creating two independent `external_body` trust assumptions for the same underlying function. While each is individually correct, duplicating trust assumptions makes it harder to track which assumptions are in play.
- **Suggested Fix:** Extract `clock_now()` into a shared utility module (e.g., `kernel/pm/clock.rs`) so there is a single `external_body` declaration for this function.

- **Location:** `wf()` spec in `sleeping.spec.rs`, lines 136-139
- **Description:** The well-formedness predicate requires `alarm.unwrap() >= 0` when `alarm.is_some()`, which correctly models that `SystemTime` is non-negative. However, no upper bound is imposed. While this is technically fine (timestamps can be arbitrarily large), the real `SystemTime` is bounded by the underlying representation. This is a minor model imprecision, not a soundness issue.
- **Suggested Fix:** No immediate action required. If future proofs need bounded arithmetic, consider adding an upper bound constraint.

### Low

- **Location:** `join_cond()` omission — mentioned in exec file header (lines 43-49)
- **Description:** `join_cond()` from the original is entirely omitted. The justification is well-documented: it returns an opaque `Condvar` that Verus cannot model. Since `join_cond()` is read-only and no `SleepingThread` method modifies the condvar field, condvar identity is trivially preserved across the sleeping state lifetime. This is a sound architectural decision with appropriate documentation.
- **Suggested Fix:** None needed currently. The TODO to model condvar identity if Verus gains opaque token types is appropriate.

- **Location:** `spec_valid_reason()` in `sleeping.spec.rs`, lines 149-151
- **Description:** The valid reason predicate uses hardcoded constants 0 and 1 via `INTERRUPT_REASON_KILLED()` and `INTERRUPT_REASON_TIMED_OUT()`. These correspond to the Rust enum `InterruptReason { Killed, TimedOut }` by positional convention. The mapping is correct for the current enum definition but is not machine-verified against the actual enum discriminants. The TODO at lines 69-76 acknowledges this.
- **Suggested Fix:** When the `InterruptReason` module is verified, add a cross-module lemma confirming the enum↔int correspondence.

- **Location:** Proof lemmas in `sleeping.proof.rs`, lines 99-107 (wakeup lemmas)
- **Description:** The wakeup proof lemmas construct `ReadyThread { state: self.state, admission_time: 0int }` with a hardcoded `admission_time: 0`. This is valid for the proof (which only needs to reason about the structural relationship), but it creates a minor disconnect with the exec implementation where `admission_time` comes from `clock_now()`. The lemma's `ensures` don't claim anything about `admission_time`, so this is sound.
- **Suggested Fix:** No fix needed. The hardcoded value is a proof witness, not an executable claim.

## Positive Observations

- **Zero `assume` statements.** The entire verification (33 conditions) passes without any `assume` directives, indicating no gaps are papered over.
- **Comprehensive function coverage.** All 10 public functions from the original are accounted for: `from_state`, `wakeup`, `interrupt`, `id`, `thread_state`, `thread_state_mut`, `join_cond`, `alarm`, `set_thread_data_area`, `get_thread_data_area`. The two that cannot be modeled (`join_cond` and `thread_state_mut`) have thorough trust boundary documentation.
- **Strong postconditions on state transitions.** Both `wakeup()` and `interrupt()` verify identity preservation, well-formedness, mutex accounting (per-address frame conditions via universal quantifiers), and drop safety. These are the essential correctness properties for kernel thread state transitions.
- **Clean spec/proof/exec separation.** The three-file split is well-executed: specs are purely declarative, proofs are standalone lemmas, and exec code is clean. The `include!` macro pattern works well for this codebase.
- **Thorough documentation.** Trust boundaries, cross-module obligations, verification model decisions, and out-of-scope items are all explicitly documented. The CROSS-MODULE-CHECK comments (e.g., lines 166-171 of exec) create a clear audit trail.
- **Round-trip verification for TDA.** The `set_thread_data_area` / `get_thread_data_area` pair is verified at both the `ThreadState` level and the `SleepingThread` level, including preservation of all unrelated fields.
- **View equality lemma.** `lemma_view_equality` provides extensional equality reasoning for `SleepingThread`, useful for compositional proofs.
- **Boundary models are minimal and well-scoped.** The `ReadyThread` and `InterruptedThread` boundary models include only what is needed for verifying `SleepingThread`'s transitions, with clear cross-module validation TODOs.

## Summary

This is a high-quality verification of the `SleepingThread` module. All 10 original functions are accounted for (8 verified, 1 external, 1 omitted-with-justification). The 33 verification conditions pass cleanly with no `assume` statements. The key correctness properties — identity preservation across state transitions, well-formedness propagation, mutex accounting consistency, drop safety, and alarm faithfulness — are all captured and proven.

The main architectural concern is the divergent boundary model strategy for `ReadyThread` across sibling modules (sleeping vs. interrupted), which will need reconciliation for cross-module composition. The `thread_state_mut()` external annotation is a necessary concession to Verus limitations, with good documentation and a verified alternative (`set_thread_data_area`). The duplicated `clock_now()` external_body across modules is minor tech debt.

Overall, this verification achieves its stated goals well and provides meaningful assurance about the state transition correctness of the sleeping thread lifecycle.
