# Review: ready (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** ReadyThread::join_cond (missing in exec/spec/proof)
  - **Status:** **Not fixed.**
  - **Evidence:** The trust-boundary docs still state that `join_cond()` is omitted (ready.rs:31-33; ready.spec.rs:20-26).
  - **Impact:** The public API is not fully modeled, leaving synchronization behavior unchecked.
  - **Suggested Fix:** Add a boundary model (e.g., opaque `Condvar`) with an `external_body` spec that returns the stored condvar without mutating state.

- **Location:** ReadyThread::thread_state_mut (exec, `#[verifier::external]`)
  - **Status:** **Not fixed.**
  - **Evidence:** The method remains `#[verifier::external]` with only comment-level obligations (ready.rs:495-531).
  - **Impact:** Callers can still mutate the state without any machine-checked guarantees for `wf()` or identity preservation.
  - **Suggested Fix:** Provide a verified wrapper with explicit `requires`/`ensures`, or replace with verified forwarding methods for all mutation paths and restrict/remove the escape hatch.

### Medium
- **Location:** ReadyThread::run (exec/spec)
  - **Status:** **Not fixed.**
  - **Evidence:** The model still omits the `*mut ContextInformation` return value (ready.rs:441-446).
  - **Impact:** Correctness of the context pointer used for low-level context switching remains unverified.
  - **Suggested Fix:** Add an abstract handle/token for the context pointer with a trusted postcondition tying it to the thread state.

- **Location:** ReadyThread::new/from_state (exec/spec)
  - **Status:** **Not fixed.**
  - **Evidence:** The constructor still omits `ContextInformation` and `FpuState` (ready.rs:251-255).
  - **Impact:** HAL state initialization/preservation remains outside the verification model and could hide invariants.
  - **Suggested Fix:** Introduce opaque ghost fields or abstract predicates for context/FPU well-formedness and thread those through specs.

### Low
- **Location:** clock_now()/admission_time specs (exec/spec)
  - **Status:** **Not fixed.**
  - **Evidence:** `clock_now()` still only ensures non-negativity (ready.rs:69-77).
  - **Impact:** No monotonicity or ordering guarantees for admission time if scheduling relies on them.
  - **Suggested Fix:** Strengthen the time model if admission ordering matters for correctness proofs.

- **Location:** EXIT_STATUS_INTERRUPTED() (spec)
  - **Status:** **Partially addressed, still unresolved.**
  - **Evidence:** The constant remains hard-coded to `4` with documentation (ready.spec.rs:68-76). The new exec bridge `exit_status_interrupted_value()` does not link this to the real error module.
  - **Impact:** Potential drift between spec constant and actual `ErrorCode::Interrupted` value.
  - **Suggested Fix:** Tie the constant to a shared spec in the error module or prove equality with the real enum value.

## Positive Observations
- Trust-boundary documentation is clearer, and the exec/spec bridge for exit status is explicit.
- Core state-transition specs remain consistent and well-structured.

## Summary
Most of the prior issues remain: `join_cond()` is still omitted, the external mutable accessor is unchecked, and HAL/context pointer modeling is still absent. Documentation improved, but the verification coverage is still incomplete in the same places as before. The prover’s claimed fixes are largely *not* realized in code.
