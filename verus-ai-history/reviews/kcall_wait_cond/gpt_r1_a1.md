# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Missing mutex/condvar postconditions for non-success returns**
  - **Location:** `wait_cond_model` postconditions (exec, `wait_cond.rs`).
  - **Description:** The model only guarantees `spec_mutex_released`, `spec_cond_ref_released`, and `spec_mutex_reacquired` when `spec_is_success(result)` holds. In the real code, if `get_cond` succeeds and `cond.wait` returns `TimedOut/Killed/Generic`, the continuation pipeline still succeeds and the mutex is reacquired before returning those error results; similarly, when `get_cond` fails but the continuation steps succeed, the mutex is reacquired before returning `GetCondError`. The current spec omits this key safety property, making the postcondition too weak for callers that rely on the mutex being held on any “stored result” return.
  - **Suggested Fix:** Strengthen the postcondition to require reacquisition and cond-ref release whenever the function returns a stored-result variant (Success, CondWait*, GetCondError) or, equivalently, whenever the result is **not** a continuation error (`PutCondError`, `GetMutexError`, `Lock*`, `PutGuardError`) and timeout/take-guard errors are excluded. Add a lemma that characterizes “stored-result return” and use it to derive the protocol predicates.

### Medium
- **Resource invariants between get/put operations are not modeled**
  - **Location:** External-body specs in `wait_cond.rs` and uninterpreted predicates in `wait_cond.spec.rs`.
  - **Description:** The model treats `get_cond` and `put_cond` as independent outcomes, with `spec_cond_ref_released` asserted on `put_cond` success even if `get_cond` failed (and no reference was acquired). Similarly, `take_mutex_guard`/`put_mutex_guard` are not linked by an ownership invariant. This weakens the verification by allowing behaviors that violate reference counting and guard ownership, so resource-safety properties are not actually proven.
  - **Suggested Fix:** Introduce a small abstract state for condvar refcounts and mutex-guard ownership, and strengthen external-body contracts (e.g., `put_cond` success requires a prior successful `get_cond`, and `put_mutex_guard` success requires a successful `lock` and prior `take_mutex_guard`). Then prove state preservation across the pipeline.

- **Liveness/timing semantics are not captured beyond a boolean alarm flag**
  - **Location:** `cond_wait_model` in `wait_cond.rs`, and out-of-scope notes in `wait_cond.rs` header.
  - **Description:** The model only constrains `TimedOut` to occur when an alarm is present, but it does not model the actual alarm value or any eventual wakeup/timeout behavior. This means the verification cannot establish key liveness properties such as “a finite timeout eventually returns TimedOut if not signaled” or “no TimedOut occurs before the alarm time.”
  - **Suggested Fix:** Add a time-abstracted model (e.g., logical time and monotonicity) or link to a proven condvar/clock module with temporal guarantees, then lift those guarantees into `wait_cond`.

### Low
- **Architecture assumption is implicit in the model types**
  - **Location:** `wait_cond.rs` uses `u32` inputs and `USIZE_MAX_X86_32`.
  - **Description:** Equivalence to the original `usize`-based API relies on the x86-32 assumption, but the precondition is not explicitly stated on the exec model. This is fine for the current target but makes the spec less portable and could be misused on other architectures.
  - **Suggested Fix:** Add an explicit architecture precondition to `wait_cond_model` (or a compile-time guard) that enforces 32-bit `usize` when interpreting the model as equivalent to the kernel implementation.

## Positive Observations
- The control-flow equivalence is modeled precisely, especially the “stored result vs. continuation pipeline” behavior and error overriding semantics.
- Timeout parsing matches `SystemTime::new` and the InvalidArgument error mapping is validated.
- The split between exec/spec/proof is clean, with well-structured view types and lemmas for key error-propagation cases.

## Summary
The verification captures the core control-flow and error-propagation semantics, but it misses an important mutex/condvar postcondition for non-success stored-result returns and lacks explicit resource-state invariants. Strengthening these areas would better align the spec with the kernel’s expected safety guarantees while preserving the otherwise strong exec/spec equivalence.
