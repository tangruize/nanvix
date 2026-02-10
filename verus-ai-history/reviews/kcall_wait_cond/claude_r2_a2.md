# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: A

## Previous Issue Resolution

### High #1: `cond_wait_model` only receives `has_alarm: bool` — **Partially Fixed**

The prover added ghost parameters `timeout_s: Ghost<u32>` and `timeout_ns: Ghost<u32>` to `cond_wait_model` (exec line 401-402) with precondition `has_alarm ==> spec_is_finite_timeout(timeout_s@ as nat, timeout_ns@ as nat)` (exec line 405). The ghost values are threaded from `wait_cond_model` through `get_cond_and_wait_model` (exec lines 693-694) to the trust boundary.

**Verification:** The precondition creates a proof obligation at the call site ensuring the actual timeout parameters (not arbitrary values) reach the trust boundary. If someone swapped `timeout_s` and `timeout_ns` in the call, the precondition would fail in most cases (since `spec_is_finite_timeout` checks `timeout_ns < NANOS_PER_SEC()`). This is a meaningful improvement.

**Residual gap:** The postcondition of `cond_wait_model` still does not reference the ghost timeout values — it only asserts `TimedOut ==> has_alarm`. A stronger external_body contract could include an uninterpreted postcondition like `spec_alarm_value_matches(timeout_s@, timeout_ns@)`, but this is an inherent limitation of trust boundaries where the implementation is not verified. Downgraded to Low.

### Medium #1: `put_cond_model` called unconditionally without ref-acquired precondition — **Addressed via documentation**

The trust boundary T4 comment now explicitly states "Called unconditionally in the original code regardless of get_cond outcome" (exec line 66). No structural change was made. The model faithfully reproduces the original code's behavior where `put_cond` runs regardless of `get_cond` outcome, and whether that pattern is itself correct is out of scope for this module's verification. Accepted.

### Medium #2: No assertion that steps aren't reached on short-circuit — **Addressed via documentation**

Don't-care ghost state comments were improved (exec lines 646-647, 669-670). The pre-existing `lemma_take_guard_short_circuit` and `lemma_pipeline_short_circuit_timeout` already prove the semantic equivalence (all subsequent outcomes are irrelevant on short-circuit). The comments now clearly label don't-care values. No structural "step reached" flags were added, but the existing lemmas are sufficient to prove the property. Accepted.

### Medium #3: `LockTimedOut` dead variant — **Fixed**

New comprehensive `lemma_lock_timed_out_unreachable` (proof lines 929-1004) proves that for ALL combinations of step outcomes, when `lock_outcome` excludes `LoTimedOut`, the final result can never be `WaitCondResultView::LockTimedOut`. Unlike the previous `lemma_reacquisition_no_timed_out` which only covered a subset (put_cond=Ok, get_mutex=Ok), this new lemma covers all seven step outcomes universally. Doc comments added on the variant in both spec (line 182-183) and exec (lines 293-294) explicitly marking it as dead. Verified that the lemma passes (30 VCs, up from 29).

### Low #1: Parameter order mismatch — **Fixed**

Doc comment added on `take_mutex_guard_model` (exec lines 354-356): "Mutex address (reordered first since it is the exec param; original API order is `pid, tid, mutex_addr`)." Accepted.

### Low #2: Ghost state don't-care values — **Fixed**

Invalid-timeout ghost state now uses `TmgOk` instead of `TmgError { error_code: 0int }` (exec line 648). Take-guard-error ghost state now uses `GcOk` variants with clear "Don't-care values" comments (exec lines 669-676). Consistent `Ok` variants used throughout for don't-care fields, eliminating the misleading error codes.

### Low #3: Uninterpreted safety predicates need trust documentation — **Fixed**

New "Trust Assumptions" section added to module header (exec lines 71-83) listing all four uninterpreted predicates with descriptions. Exactly as suggested.

## Issues Found

### Critical

(none)

### High

(none)

### Medium

(none)

### Low

- **Location:** `cond_wait_model` postcondition (exec, lines 406-408)
- **Description:** (Residual from previous High #1, downgraded.) The ghost timeout parameters `timeout_s` and `timeout_ns` are threaded through to the trust boundary and validated by the precondition, but the postcondition doesn't reference them. The ensures clause only asserts `TimedOut ==> has_alarm`. A hypothetical external_body implementation could ignore the ghost timeout values and use a different alarm without violating the postcondition. Since this is an external_body trust boundary by design, and the precondition already ensures the correct values are passed to the boundary, this is a minor completeness gap rather than a soundness issue.
- **Suggested Fix:** Consider adding an uninterpreted postcondition predicate like `result matches CondWaitOutcomeModel::Ok ==> spec_waited_with_correct_alarm(timeout_s@ as nat, timeout_ns@ as nat)` to make the trust boundary contract more precise about value propagation. Alternatively, document this as an accepted limitation of the trust boundary.

## Positive Observations

- **All 30 verification conditions pass cleanly.** One new lemma added (from 29 to 30).
- **Comprehensive pipeline modeling.** The 7-step pipeline with stored-result semantics and continuation-override behavior remains faithfully modeled.
- **Strong exec-spec equivalence.** `wait_cond_model` postcondition ties exec result to `spec_wait_cond_result` via ghost state bundle.
- **Rich proof library.** 20+ lemmas covering error propagation, short-circuit, exhaustiveness, result preservation, dead variant unreachability, and mutex protocol properties.
- **Clean spec/proof/exec separation.** View types and spec functions in `.spec.rs`, proof lemmas in `.proof.rs`, exec models in `.rs`.
- **Mutex protocol postcondition.** `spec_mutex_released ∧ spec_cond_ref_released ∧ spec_mutex_reacquired` on stored-result returns is a valuable caller-facing safety guarantee.
- **Timeout value threading.** Ghost parameters now create a verifiable chain from user-provided timeout values to the condvar wait trust boundary.
- **Excellent documentation.** Trust Assumptions section, API mapping table, trust boundary listing, don't-care value annotations, and dead variant markers make the verification self-documenting.
- **Responsive to review.** All seven previous issues were addressed — three structurally, four via documentation. The structural fixes (ghost timeout params, comprehensive dead variant lemma, consistent don't-care values) are substantive improvements.

## Summary

The prover has substantively addressed the previous review's findings. The highest-priority issue (timeout value threading) was fixed by adding ghost parameters to `cond_wait_model` with a precondition validating the timeout values, though the postcondition could still be stronger. The `LockTimedOut` dead variant now has a comprehensive unreachability proof covering all input combinations. Documentation improvements (trust assumptions, parameter reordering notes, don't-care annotations) are thorough and helpful.

The one remaining Low-priority item is the absence of timeout value usage in `cond_wait_model`'s postcondition. This is an inherent limitation of the external_body trust boundary pattern — the ghost parameters ensure the correct values arrive at the boundary, but the boundary doesn't promise it uses them. This is a reasonable tradeoff for a verification that focuses on control-flow correctness and error-handling semantics.

Overall, this is a high-quality verification suitable for an OS kernel component. The control flow, error propagation, short-circuit semantics, continuation-override behavior, and mutex protocol properties are all rigorously verified. The verification covers the essential correctness properties of the `wait_cond` kcall.
