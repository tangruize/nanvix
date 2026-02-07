# Review: ready (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** ReadyThread::join_cond (missing in exec/spec/proof)
  - **Status:** **Not fixed.**
  - **Evidence:** `join_cond()` remains explicitly omitted in the module docs and spec trust assumptions (ready.rs:31-33, 47; ready.spec.rs:20-26).
  - **Impact:** Public API coverage is incomplete; synchronization behavior remains unchecked.
  - **Suggested Fix:** Add a boundary model for `Condvar` with an `external_body` stub specifying it returns the stored condvar without mutating state.

- **Location:** ReadyThread::thread_state_mut (exec, `#[verifier::external]`)
  - **Status:** **Not fixed.**
  - **Evidence:** Still declared `#[verifier::external]` with only comment-level obligations (ready.rs:495-531).
  - **Impact:** Callers can violate `wf()` or identity without machine-checked guarantees.
  - **Suggested Fix:** Provide verified wrappers with explicit `requires`/`ensures`, or restrict/remove the escape hatch.

### Medium
- **Location:** ReadyThread::run (exec/spec)
  - **Status:** **Not fixed.**
  - **Evidence:** The raw context pointer remains omitted from the modeled return type (ready.rs:441-446).
  - **Impact:** Context pointer correctness used by low-level context switching is unverified.
  - **Suggested Fix:** Introduce an abstract handle/token with a trusted postcondition relating it to the thread context.

- **Location:** ReadyThread::new/from_state (exec/spec)
  - **Status:** **Not fixed.**
  - **Evidence:** `ContextInformation` and `FpuState` are still elided (ready.rs:251-255).
  - **Impact:** HAL state initialization/preservation remains outside verification.
  - **Suggested Fix:** Add opaque ghost fields or predicates for context/FPU well-formedness and thread them through specs.

### Low
- **Location:** clock_now()/admission_time specs (exec/spec)
  - **Status:** **Not fixed.**
  - **Evidence:** `clock_now()` still only ensures non-negativity (ready.rs:69-77).
  - **Impact:** No monotonicity/ordering guarantees for admission time if scheduling relies on them.
  - **Suggested Fix:** Strengthen the time model if admission ordering is required for correctness.

- **Location:** EXIT_STATUS_INTERRUPTED() (spec)
  - **Status:** **Unresolved.**
  - **Evidence:** Constant remains hard-coded to `4` with only documentation (ready.spec.rs:68-76). Exec bridge does not tie to real error module.
  - **Impact:** Potential drift from `ErrorCode::Interrupted` value.
  - **Suggested Fix:** Link to a shared spec in the error module or prove equality with the real enum.

## Positive Observations
- No new regressions detected in the core transition specs.
- Trust-boundary documentation remains explicit.

## Summary
No substantive fixes landed; the same gaps persist (missing `join_cond`, unchecked `thread_state_mut`, and omitted HAL/context modeling). Verification coverage is still incomplete in the same high-risk areas.
