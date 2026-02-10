# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

- **Location:** `cond_wait_model` (exec, line 380)
- **Description:** The `cond_wait_model` external body receives only `has_alarm: bool`, discarding the actual `SystemTime` value (seconds/nanoseconds). The original code passes the full `Option<SystemTime>` to `cond.wait(alarm)`. This means the verification cannot prove that the correct timeout value is propagated to the condition variable wait — only that a timeout *exists* or not. An implementation that swapped seconds and nanoseconds, or passed a zero timeout instead of the user-specified one, would still satisfy the spec.
- **Suggested Fix:** Extend `cond_wait_model` to accept ghost parameters for `timeout_s` and `timeout_ns`, and add a postcondition linking them to the alarm used internally. Alternatively, add a ghost postcondition on `parse_timeout_model` that threads the parsed value through to the wait call via a tracked ghost token.

### Medium

- **Location:** `put_cond_model` called unconditionally (exec, lines 657-675; original line 130)
- **Description:** In the original code, `ProcessManager::put_cond(cond_addr)` is called unconditionally after the `get_cond`+`cond.wait` block, even when `get_cond` failed (meaning no condition variable reference was acquired). The model faithfully reproduces this, but the `put_cond_model` postcondition asserts `spec_cond_ref_released(cond_addr as nat)` on success without any precondition requiring that the ref was previously acquired. This could mask a double-release or release-without-acquire bug in the original. The spec should encode that `put_cond` on a non-acquired cond is either an error or a no-op.
- **Suggested Fix:** Add an uninterpreted spec predicate `spec_cond_ref_acquired(cond_addr: nat) -> bool` and make it a postcondition of `get_cond_model` on success. Then require it as a precondition of `put_cond_model`, or document in the spec that `put_cond` is safe to call regardless. This would surface potential ref-count bugs.

- **Location:** `spec_wait_cond_result` spec function (spec, lines 320-386) — no `put_cond` call modeled when `take_mutex_guard` fails
- **Description:** In the original code, when `take_mutex_guard` fails, the function returns early via `?` on line 112, *before* reaching `get_cond`, `put_cond`, or the continuation pipeline. The spec correctly models this as a short-circuit. However, the verification does not assert that `put_cond` is NOT called when `take_mutex_guard` fails. If the original code were refactored to call `put_cond` on all paths (a plausible mistake), the model would not catch the discrepancy unless the spec explicitly disallows it.
- **Suggested Fix:** Add a proof lemma asserting that when `take_mutex_guard` fails, the `put_cond`, `get_mutex`, `lock`, and `put_guard` outcomes are irrelevant (already partially covered by `lemma_take_guard_short_circuit`, but the step-not-reached semantic is only implicit). Consider adding ghost "step reached" flags to `WaitCondGhostState`.

- **Location:** `LockTimedOut` variant retained in `WaitCondResultView` and `LockOutcomeModel` (spec line 183, exec line 279)
- **Description:** The `mutex_lock_model` external body guarantees `!matches!(result, LockOutcomeModel::TimedOut)` (line 428), making the `LockTimedOut` path unreachable. However, `WaitCondResultView::LockTimedOut` is a first-class result variant and appears in `spec_is_lock_error`. While the model handles this correctly (the branch is vacuously true), a reader may wonder whether `LockTimedOut` is a genuine outcome. The `lemma_reacquisition_no_timed_out` proof partially addresses this but only for a subset of input combinations.
- **Suggested Fix:** Add a comprehensive lemma proving `!matches!(result, WaitCondResultView::LockTimedOut)` for ALL valid inputs (where `mutex_lock_model` was called), or add a doc comment on `LockTimedOut` explicitly marking it as a dead variant retained for exhaustiveness.

### Low

- **Location:** `take_mutex_guard_model` parameter order (exec, line 342)
- **Description:** The original API is `take_mutex_guard(pid, tid, mutex_addr)`, but the model reorders to `take_mutex_guard_model(mutex_addr, pid, tid)`. While pid/tid are Ghost params and this doesn't affect correctness, it diverges from the original API ordering and could confuse readers doing manual equivalence checking.
- **Suggested Fix:** Match the original parameter order or add a comment noting the deliberate reordering.

- **Location:** Ghost state don't-care values (exec, lines 611-618, 631-638)
- **Description:** When an early return occurs (e.g., timeout parse failure), the ghost state is populated with arbitrary values (e.g., `TmgError { error_code: 0int }` at line 611 for the tmg field when timeout parsing fails and take_mutex_guard was never called). While the spec correctly ignores these via short-circuit, using `TmgError` with a fake error code 0 is misleading — it suggests take_mutex_guard was called and failed with code 0.
- **Suggested Fix:** Use `TmgOk` or a clearly-named sentinel for don't-care ghost state fields, and add a brief comment. Alternatively, define don't-care constants (e.g., `GHOST_DONT_CARE_TMG`) to make intent explicit.

- **Location:** Spec file (spec, lines 478-500) — uninterpreted safety predicates
- **Description:** The safety precondition predicates (`spec_caller_is_not_kernel_process`, `spec_caller_holds_no_resources`, `spec_caller_no_pm_reference`, `spec_is_currently_running`) are declared as `uninterp` with no axioms constraining them. This is correct for modularity, but means the verification cannot detect if the caller violates these preconditions. This is acceptable for a kcall boundary but should be documented as a trust assumption.
- **Suggested Fix:** The existing doc comments are adequate. Consider adding a "Trust Assumptions" section in the module-level documentation explicitly listing that these predicates are assumed correct by the caller.

## Positive Observations

- **Comprehensive pipeline modeling.** The 7-step pipeline (take_mutex_guard → get_cond → cond.wait → put_cond → get_mutex → lock → put_guard) is modeled faithfully with the critical "stored result" semantics correctly captured. The distinction between short-circuit errors (timeout, take_mutex_guard) and continuation-override errors (put_cond through put_guard) is precisely specified.
- **Strong exec-spec equivalence.** The `wait_cond_model` postcondition directly ties the exec result to `spec_wait_cond_result` applied to ghost step outcomes, providing end-to-end correspondence.
- **Rich proof library.** 20+ lemmas covering error propagation, short-circuit, exhaustiveness, result preservation, and mutex protocol properties. The `lemma_success_requires_all_steps` biconditional is particularly strong.
- **Clean spec/proof/exec separation.** View types and spec functions in `.spec.rs`, proof lemmas in `.proof.rs`, exec models and external bodies in `.rs`. The `include!` mechanism keeps them logically unified while physically separated.
- **Mutex protocol postcondition.** The ensures clause on `wait_cond_model` proving `spec_mutex_released ∧ spec_cond_ref_released ∧ spec_mutex_reacquired` on stored-result returns is a valuable safety property for callers.
- **External body postconditions are well-designed.** Each trust boundary has meaningful postconditions (e.g., `cond_wait_model` ensures TimedOut requires has_alarm; `mutex_lock_model` ensures no TimedOut).
- **Verification passes cleanly.** All 29 verification conditions pass with no errors.
- **Excellent documentation.** The API mapping table, trust boundary listing, and verified/out-of-scope property lists in the module header are exemplary.

## Summary

This is a thorough and well-structured verification of the `wait_cond` kernel call's control flow and error handling. The 7-step pipeline is faithfully modeled, the stored-result-vs-continuation semantics are correctly captured, and the proof library provides strong guarantees about error propagation, short-circuit behavior, and the mutex release/reacquire protocol.

The primary gap is that the actual timeout value is not threaded through the verification — only the boolean "has alarm" flag reaches `cond_wait_model`. This means the spec cannot detect bugs where the wrong timeout value is passed to the condition variable. Addressing this (High priority) would elevate the verification from control-flow correctness to full functional correctness of the timeout propagation path.

The medium-priority items around `put_cond` being called without a matching `get_cond` success, and the dead `LockTimedOut` variant, are minor spec-hygiene issues that don't affect soundness but could improve the model's ability to catch future regressions. Overall, this is high-quality verification work suitable for an OS kernel component.
