# Review: semaphore (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

None.

### High

1. **View implementation hardcodes `waiters: 0`, disconnecting ghost state from exec**
   - **Location:** `semaphore.spec.rs`, `View for Semaphore` (line 321–323)
   - **Description:** The `View` trait implementation always produces `SemaphoreView { value: self.value as nat, waiters: 0 }`. This means `self@.waiters` is always 0 for any exec `Semaphore`, making the `wf()` constraint `self@.waiters > 0 ==> self@.value == 0` vacuously true at the exec level. All proof lemmas about waiters (`lemma_down_blocking_preserves_wf`, `lemma_wake_preserves_wf`, `lemma_up_wake_cycle`, `lemma_all_waiters_eventually_served`) operate on manually constructed `SemaphoreView` values, not on any exec `Semaphore`'s view. The blocking protocol proofs are therefore disconnected from the exec code — they prove properties of a spec-level state machine that is never linked back to the exec struct through `@`.
   - **Suggested Fix:** Consider adding a ghost field (e.g., `tracked waiters: Ghost<nat>`) to the exec `Semaphore` struct so that `view()` can include a non-trivial waiter count. Alternatively, document more explicitly that the waiter proofs are pure state-machine reasoning with no exec anchor, and consider whether the proof effort on waiters adds verifiable value beyond documentation.

2. **Error paths from `up()` not modeled**
   - **Location:** `semaphore.rs`, `up()` (line 335–348); original `semaphore.rs` line 154–157
   - **Description:** The original `up()` returns `Result<(), Error>` because `self.sleeping.notify_first().map(|_awakened| ())` can fail (e.g., if the condvar notification encounters an error). The verified `up()` returns `()` unconditionally, modeling only the success path. If `notify_first()` fails in the original, the semaphore value has already been incremented but no waiter is woken — a state that the verified model cannot represent or reason about. This is a semantic gap where the verified model is strictly more permissive than the original.
   - **Suggested Fix:** Model `up()` as returning a `bool` or `Result`-equivalent that distinguishes the notification success/failure paths. At minimum, add a spec-level note that the model assumes `notify_first()` always succeeds, and add this to the trust assumptions.

3. **Error path from `down()` not modeled**
   - **Location:** `semaphore.rs`, `down()` (line 215–227); original `semaphore.rs` line 83–101
   - **Description:** The original `down()` returns `Result<(), SleepError>` because `self.sleeping.wait(None)?` can fail (the thread may be woken with an error, e.g., signal interruption). The verified `down()` returns `()` with a precondition that the semaphore is available, making the blocking-then-error case unrepresentable. While the `down_or_block()` function models the blocking decision, it does not model the `SleepError` failure path either.
   - **Suggested Fix:** Add a `DownOutcome::SleepFailed` variant or separate spec-level transition modeling the `wait()` failure case. Document in the trust assumptions that `SleepError` propagation is out of scope.

### Medium

1. **`down()` precondition shifts availability burden to caller**
   - **Location:** `semaphore.rs`, `down()` requires clause (line 218)
   - **Description:** The `requires old(self).spec_is_available()` precondition demands the caller proves the semaphore has resources before calling `down()`. In the original, `down()` is a blocking call — the caller does *not* need to know the value is positive; the function loops until it succeeds. While `down_or_block()` addresses this by modeling both paths, the standalone `down()` function has a strictly stronger precondition than the original, which may mislead verification consumers into thinking callers must establish availability.
   - **Suggested Fix:** Consider deprecating the standalone `down()` in favor of `down_or_block()` as the primary verified entry point, or rename `down()` to `down_available()` to clarify it models only the instant-success path.

2. **`spec_wake` precondition allows `value > 1` but real wake only occurs from `value == 1`**
   - **Location:** `semaphore.spec.rs`, `spec_wake()` (line 200–206)
   - **Description:** The `recommends` clause allows `view.value > 0`, but in the actual protocol, `spec_wake` is applied after `up()` increments value from 0 to 1 (since `wf()` requires `waiters > 0 ==> value == 0`, and `up()` adds 1). The spec function is more permissive than the protocol allows — it can be called with `value == 5, waiters == 3` which is an unreachable state. The `lemma_wake_preserves_wf` correctly constrains to `value == 1`, but the spec function itself is looser.
   - **Suggested Fix:** Strengthen `spec_wake` recommends to `view.value == 1` or add `Semaphore::spec_wf(view)` as a recommends clause to prevent misuse.

3. **`spec_condvar_wake_after_notify` is a dead interface contract**
   - **Location:** `semaphore.spec.rs`, lines 227–235; doc comment lines 220–226
   - **Description:** The doc comment at line 222–226 explicitly acknowledges that this spec "is not imported by the condvar module" and "changes to the condvar implementation will not trigger a verification failure here." This means the condvar trust assumption (T2) is stated but never cross-checked. If the condvar module's behavior diverges from this spec, the semaphore proofs remain green — a silent soundness gap at the module boundary.
   - **Suggested Fix:** Create a shared trait or spec module that both semaphore and condvar import, so changes to condvar behavior would break semaphore verification if the contract is violated.

### Low

1. **`pub value: usize` field violates Nanvix coding standards**
   - **Location:** `semaphore.rs`, line 164
   - **Description:** Nanvix coding standards require struct fields to be private with getter/setter access. The field is `pub` due to Verus tooling constraints (documented in line 160). This is a minor style deviation.
   - **Suggested Fix:** No immediate fix needed; this is a known Verus limitation. Consider filing a Verus issue if not already tracked.

2. **Many proof lemmas are trivially discharged (empty body)**
   - **Location:** `semaphore.proof.rs`, multiple lemmas (e.g., `lemma_new_is_correct`, `lemma_state_is_total`, `lemma_down_up_roundtrip_by_postconditions`, etc.)
   - **Description:** Approximately 20 of the 25+ lemmas have empty bodies, meaning Z3 proves them with zero guidance. While this demonstrates the specs are well-designed, it also suggests the properties may be simpler than they appear (some are essentially tautologies over nat arithmetic). The proof effort is concentrated in `lemma_all_waiters_eventually_served` (inductive) and `lemma_up_wake_cycle`.
   - **Suggested Fix:** No fix needed, but consider whether some trivially-true lemmas (e.g., `lemma_up_monotonic`: `(v + 1) > v`) add verification value beyond documentation.

3. **`spec_after_n_up_wake_cycles` does not enforce `spec_wf` at intermediate steps**
   - **Location:** `semaphore.spec.rs`, `spec_after_n_up_wake_cycles()` (line 254–264)
   - **Description:** The recursive spec function applies up-wake cycles without requiring `spec_wf` at each intermediate state. The `lemma_all_waiters_eventually_served` does verify this inductively, but the spec function itself could be called with ill-formed intermediate views. This is minor since the function is only used in proofs with proper preconditions.
   - **Suggested Fix:** Add a `recommends Semaphore::spec_wf(view)` clause to the spec function.

## Positive Observations

- **Zero trust holes:** No `assume`, `external_body`, or `trusted` annotations. All 43 verification conditions pass cleanly.
- **Exceptional documentation:** The module-level documentation is among the best I've seen for verified code — it clearly describes the verification model, API mapping, divergences, trust assumptions, refinement argument, and verification scope. A reader can understand exactly what is and isn't proven.
- **Clean spec/proof/exec separation:** The three-file split is well-organized. Spec functions are in `semaphore.spec.rs`, proofs in `semaphore.proof.rs`, exec code in `semaphore.rs`. No proof leaks into exec code.
- **CallerContext ghost modeling:** Encoding `unsafe` preconditions (interrupts disabled, not kernel process, no held resources) as ghost `CallerContext` is a creative approach that brings unsafe obligations into the verification scope.
- **Inductive waiter draining proof:** `lemma_all_waiters_eventually_served` with the recursive `spec_after_n_up_wake_cycles` is a meaningful property proving that w up-wake cycles drain all w waiters — a non-trivial liveness-adjacent result.
- **`down_or_block()` addition:** This function bridges the exec and spec layers for the blocking decision, providing a more faithful model of the original `down()` than the preconditioned `down()` alone.
- **Overflow guard on `up()`:** The original `fetch_add(1, SeqCst)` can silently overflow; the verified model makes this an explicit precondition (`value < usize::MAX`), surfacing a real bug class.

## Summary

The semaphore verification provides a solid sequential model of the core state machine with thorough documentation and no soundness holes (no assume/external_body). All four original functions (`new`, `down`, `try_down`, `up`) are covered, plus useful additions (`down_or_block`, `get_value`, `is_available`).

The main weakness is the disconnect between exec-level and spec-level reasoning about the blocking protocol: the `View` implementation hardcodes `waiters: 0`, so all waiter-related invariants and proofs operate on free-floating `SemaphoreView` values rather than being grounded in exec state. This means the blocking protocol proofs (`spec_down_blocking`, `spec_wake`, waiter draining) are essentially standalone state-machine proofs that cannot be violated by changes to the exec code.

Secondary gaps include unmodeled error paths (`SleepError` from `down()`, `Error` from `up()`'s `notify_first()`) and the dead condvar interface contract that cannot detect condvar behavior changes. These are transparently documented but represent real verification gaps.

Overall, this is a well-crafted verification that proves sequential protocol correctness within its stated scope. The grade reflects that the core exec functions are verified without trust holes, but the ghost waiter state is disconnected from exec, error paths are omitted, and the concurrent-to-sequential refinement remains informal.
