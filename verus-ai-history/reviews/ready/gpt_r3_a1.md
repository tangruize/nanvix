# Review: ready (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** ReadyThread::join_cond (missing in exec/spec/proof)
  - **Description:** The original `join_cond()` public method is omitted from the verified module, so coverage is incomplete and its behavior is unchecked.
  - **Suggested Fix:** Add a verified stub or boundary model for `Condvar` and specify that it returns the underlying join condvar without mutating state (or mark it `external_body` with explicit postconditions).
- **Location:** ReadyThread::thread_state_mut (exec, `#[verifier::external]`)
  - **Description:** The mutable accessor is fully external with no machine-checked pre/postconditions, so callers can silently violate `wf()` or identity invariants, undermining soundness in core scheduling code.
  - **Suggested Fix:** Replace with verified forwarding methods for every needed mutation (like the provided interrupt/mutex helpers), and/or provide a trusted wrapper with explicit `requires`/`ensures` about `wf()` and `spec_id()` preservation.

### Medium
- **Location:** ReadyThread::run (exec/spec)
  - **Description:** The verified model omits the `*mut ContextInformation` return value, so the proof does not cover correctness of the context pointer (aliasing/validity) used for low-level context switching.
  - **Suggested Fix:** Introduce an abstract token/handle for the context pointer and specify that it corresponds to the thread’s context buffer, or add a trusted postcondition capturing pointer validity.
- **Location:** ReadyThread::new/from_state (exec/spec)
  - **Description:** The model drops `context: ContextInformation` and `fpu_state: FpuState` entirely, so initialization and preservation of those HAL fields are unverified and could hide required invariants.
  - **Suggested Fix:** Add opaque ghost fields or abstract predicates for context/FPU state well-formedness, and require/ensure those predicates through constructors and transitions.

### Low
- **Location:** clock_now()/admission_time specs (exec/spec)
  - **Description:** `clock_now()` only ensures non-negativity; the spec does not capture that `admission_time` equals the time of admission or any monotonicity, which may be needed for scheduling properties.
  - **Suggested Fix:** Strengthen the time model (e.g., monotonic clock or a relation between successive admissions) if scheduling correctness relies on it.
- **Location:** EXIT_STATUS_INTERRUPTED() (spec)
  - **Description:** The status constant is hard-coded to `4` with a cross-module note but no linkage to the actual `ErrorCode` definition, risking drift.
  - **Suggested Fix:** Tie the constant to a shared spec in the error module or add a lemma that proves equality to the real `ErrorCode::Interrupted` value.

## Positive Observations
- Core transitions (`new`, `from_state`, `run`, `terminate`) have clear specs preserving identity, mutex accounting, and drop safety.
- The split between exec/spec/proof is clean and documented, with explicit trust-boundary notes and cross-module obligations.
- Additional verified helpers for interrupt and mutex management reduce reliance on unsafe mutable access.

## Summary
Verification is strong for the main state transitions but misses full coverage (notably `join_cond`) and relies on an unchecked mutable escape hatch. The model also omits the context pointer and HAL fields, leaving important low-level correctness unproven. Addressing these gaps would raise confidence to A-level.
