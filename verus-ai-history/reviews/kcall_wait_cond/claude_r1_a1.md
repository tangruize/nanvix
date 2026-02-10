# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: B-

## Issues Found

### Critical

- **Location:** `wait_cond_model` (exec, lines 538-546) and `spec_wait_cond_result` (spec, lines 283-286)
- **Description:** Semantic divergence from original when `get_cond` fails. In the original code (lines 109-123), the `get_cond` result is stored in `result`, and the pipeline **continues unconditionally** — `put_cond` (line 123), `get_mutex` (line 126), `mutex.lock` (line 127), and `put_mutex_guard` (line 128) all execute via `?`, with `result` returned at the very end (line 130). In the model and spec, `get_cond` failure causes an **immediate return** (`GetCondError`), skipping all subsequent steps. This means:
  - If `get_cond` fails AND `put_cond` also fails, the original returns the `put_cond` error (via `?`) but the model returns the `get_cond` error.
  - If `get_cond` fails but `put_cond` succeeds and `get_mutex` fails, the original returns the `get_mutex` error but the model returns the `get_cond` error.
  - Generally, any subsequent `?`-based error can override the stored `result`, which the model doesn't capture.
- **Suggested Fix:** The exec model should store the `get_cond` outcome and continue the pipeline (put_cond, get_mutex, lock, put_guard) regardless, only returning the stored `get_cond` error if all subsequent steps succeed. The spec's `spec_wait_cond_result` must be updated accordingly — when `get_cond` fails, the nested match should continue into `put_cond_outcome`, `get_mutex_outcome`, etc., and only fall through to `GetCondError` at the very end (analogous to how `cond_wait_outcome` is currently handled at the bottom of the spec).

### High

- **Location:** `wait_cond_model` (exec, lines 496-509)
- **Description:** No exec-spec equivalence proof. The postcondition of `wait_cond_model` is only `ret.0.spec_view() == ret.1@`, which trivially holds because the ghost value is constructed from the exec result. There is no postcondition connecting the model's output to `spec_wait_cond_result`. The spec function and exec model could compute different results for the same inputs without any verification failure. This is the central correctness property — that the exec model faithfully implements the spec — and it is not proven.
- **Suggested Fix:** Add an `ensures` clause to `wait_cond_model` that captures the step outcomes (e.g., via ghost variables tracking each `external_body` call result) and asserts `ret.1@ == spec_wait_cond_result(timeout_s as nat, timeout_ns as nat, tmg_view, gc_view, cw_view, pc_view, gm_view, lo_view, pg_view)`. Alternatively, write a separate proof lemma that demonstrates equivalence by case-splitting on all possible step outcomes.

### Medium

- **Location:** `wait_cond_model` (exec, line 496) and `take_mutex_guard_model` (exec, line 338)
- **Description:** The `pid` and `tid` parameters are absent from the model. The original `wait_cond` takes `pid: ProcessIdentifier` and `tid: ThreadIdentifier` and passes them to `ProcessManager::take_mutex_guard(pid, tid, mutex_addr)`. The model's `wait_cond_model` omits these entirely, and `take_mutex_guard_model` only takes `mutex_addr`. This means the model cannot verify that the correct process/thread identity is passed to the mutex guard release, which is a safety-relevant property (wrong pid/tid could release another thread's guard).
- **Suggested Fix:** Add `pid: u32` and `tid: u32` parameters to `wait_cond_model` and `take_mutex_guard_model`. Add spec predicates connecting pid/tid to ownership (e.g., `spec_thread_holds_mutex_guard(pid, tid, mutex_addr)`).

- **Location:** Uninterpreted spec predicates (spec, lines 408-436)
- **Description:** `spec_mutex_released`, `spec_mutex_reacquired`, and `spec_cond_ref_released` are defined as uninterpreted predicates and appear in `external_body` postconditions, but are never referenced in any proof lemma or in the main spec function `spec_wait_cond_result`. They represent important state transitions (mutex release/reacquisition protocol, reference counting) but have no connection to the verified correctness properties. They are essentially dead code from a proof perspective.
- **Suggested Fix:** Either (a) enrich `spec_wait_cond_result` or `wait_cond_model` postconditions to assert these state predicates (e.g., "on success, `spec_mutex_released(mutex_addr)` was true before `spec_mutex_reacquired(mutex_addr)`"), or (b) write separate proof lemmas that use these predicates to verify the mutex release-before-wait-before-reacquisition protocol.

### Low

- **Location:** Safety preconditions (spec, lines 407-421)
- **Description:** `spec_wait_cond_safety_preconditions` is defined and has a trivial well-formedness lemma (`lemma_safety_preconditions_well_formed`) but is never used as a precondition on `wait_cond_model` or connected to any correctness property. The original function's `unsafe` contract specifies three safety requirements, but the model doesn't require or leverage them.
- **Suggested Fix:** Add `requires spec_wait_cond_safety_preconditions(pid as nat, tid as nat)` to `wait_cond_model` (once pid/tid are added) to make the safety contract part of the verified precondition.

- **Location:** `parse_timeout_model` (exec, line 454)
- **Description:** The model uses `u32` for timeout parameters while the original uses `usize`. The architectural equivalence (`usize` = `u32` on x86-32) is asserted in `lemma_architecture_guard` but not enforced as a precondition. While the `requires` clause on `wait_cond_model` checks `timeout_s as nat <= USIZE_MAX_X86_32()`, this is trivially true for `u32` and doesn't actually constrain anything. The connection between the model's type choice and the original's type is documented but not formally proven as a refinement.
- **Suggested Fix:** This is acceptable given the x86-32-only target. No change strictly needed, but a comment in the `requires` clause noting the triviality would improve clarity.

- **Location:** `LockOutcomeModel::TimedOut` handling (exec, lines 576-579)
- **Description:** The model handles `LockOutcomeModel::TimedOut` in the match (step 7), even though the `mutex_lock_model` postcondition guarantees `!matches!(result, LockOutcomeModel::TimedOut)`. This dead branch exists in the model but can never be reached. While not incorrect (Verus handles it fine), it adds unnecessary complexity. The `LockOutcomeView::LoTimedOut` variant similarly exists in the spec.
- **Suggested Fix:** Consider removing the `TimedOut` variant from `LockOutcomeModel`/`LockOutcomeView` since the postcondition proves it impossible, or add a `proof { }` block asserting unreachability at that branch.

## Positive Observations

- **Thorough pipeline modeling.** All seven external dependency steps are individually modeled with appropriate `external_body` trust boundaries and postconditions (especially `cond_wait_model` ensuring TimedOut requires an alarm, and `mutex_lock_model` ruling out TimedOut with infinite wait).
- **Comprehensive proof lemmas.** The proof file contains 14 lemmas covering error propagation, short-circuit behavior, result exhaustiveness, result preservation, and the reacquisition-no-timeout property. The `lemma_success_requires_all_steps` biconditional is particularly valuable.
- **Clean separation.** Spec/proof/exec are properly split. View types provide clean abstraction boundaries. The documentation header in the exec file is exceptionally detailed, with clear API mapping table and trust boundary documentation.
- **Timeout parsing is fully verified.** The `parse_timeout_model` function is pure exec (no external_body) with strong postconditions, and the proof file thoroughly covers infinite, finite, and invalid cases.
- **All 21 verification conditions pass** with no `assume` statements in user code.

## Summary

The verification provides good structural coverage of the `wait_cond` kernel call's multi-step pipeline, with well-designed abstractions and thorough proof lemmas for error propagation and short-circuit behavior. However, there are two significant gaps:

1. **Critical semantic divergence:** The model/spec incorrectly short-circuit on `get_cond` failure, when the original code continues the full pipeline and returns the stored result only if all subsequent steps succeed. This affects the verified error-handling behavior in a way that could mask real bugs (e.g., if put_cond fails after get_cond fails, the original returns a different error than what the model predicts).

2. **Missing exec-spec linkage:** The exec model's postcondition doesn't connect to the spec function, so there's no machine-checked proof that the implementation matches the specification. The proofs operate on the spec function alone, and the exec model operates independently.

Fixing the critical divergence requires restructuring both the exec model and the spec function to match the original's "store result, continue pipeline, return stored result" pattern. Fixing the exec-spec linkage requires threading ghost state through the model to capture step outcomes and asserting equivalence with `spec_wait_cond_result`.
