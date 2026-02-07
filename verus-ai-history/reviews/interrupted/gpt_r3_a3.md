# Review: interrupted (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedThread::thread_state_mut` (exec, `verus/split/kernel/pm/thread/interrupted.rs`)
  **Status:** Not fixed.
  **Description:** The method is still `#[verifier::external]` and returns `&mut ThreadState` with no machine-checked postconditions. This remains an unverified escape hatch that lets callers violate `wf()` or identity invariants. Documentation and call-site notes do not enforce the invariant, so verification is still conditionally sound.
  **Evidence:** The function remains external with no `ensures` (exec lines ~206–243). The only checked call sites are in unverified kernel code; `ThreadRefMut::Interrupted` delegates to it and `process/manager/mod.rs` uses it for `fpu_state_mut()`, but nothing prevents other mutations.
  **Suggested Fix:** Refactor to verified setters or a wrapper with explicit postconditions once `&mut T` returns are supported; otherwise, restrict usage and re-establish `wf()`/identity at all call sites.

### Medium
- **Location:** `join_cond` (missing; original `src/kernel/src/pm/thread/interrupted.rs`, no exec/spec/proof equivalent)
  **Status:** Not fixed.
  **Description:** `join_cond()` is still omitted and explicitly marked out-of-scope. This leaves join synchronization behavior outside the verification boundary, so the verified module remains incomplete relative to the original API surface.
  **Suggested Fix:** Provide a minimal boundary model for `Condvar` and a `join_cond()` spec (or `external_body`) that ties the return value to the underlying `ThreadState` identity.

### Low
- **Location:** `ReadyThread` boundary model (exec/spec in `interrupted.rs` / `interrupted.spec.rs`)
  **Status:** Not fixed (documented).
  **Description:** The boundary model still omits `admission_time` set by the real `ReadyThread::from_state`. The documentation clarifies this is out-of-scope, but the model remains non-equivalent for scheduling/timing properties.
  **Suggested Fix:** If scheduling correctness is in scope, extend the model or verify `ReadyThread` itself and compose guarantees here.

## Positive Observations
- The documentation continues to be clear about modeling choices and trust boundaries.
- Core resume transition properties remain well specified and proven.

## Summary
No substantive fixes were made to the previously reported gaps. `thread_state_mut` remains an unverified mutability escape hatch, `join_cond` is still missing, and the ReadyThread boundary model still omits `admission_time`. Verification therefore remains incomplete and not fully sound for the original API surface.
