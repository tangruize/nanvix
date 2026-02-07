# Review: interrupted (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedThread::thread_state_mut` (exec, `verus/split/kernel/pm/thread/interrupted.rs`)
  **Status:** Not fixed.
  **Description:** The method remains `#[verifier::external]` and returns `&mut ThreadState` with no machine-checked postconditions. This still allows arbitrary mutation of a core state object, so callers can violate `wf()` or identity invariants. The added documentation and call-site notes do not enforce the invariant; verification remains conditionally sound.
  **Evidence:** The function is still external with no `ensures` (exec lines ~208–243). Call sites are in unverified kernel code (e.g., `process/manager/mod.rs:1154,1169`) and only use `fpu_state_mut()`, which is *not* enforced by Verus.
  **Suggested Fix:** Refactor to verified setters or a wrapper with explicit postconditions once `&mut T` returns are supported; otherwise, restrict usage and re-establish `wf()`/identity at all call sites.

### Medium
- **Location:** `join_cond` (missing; original `src/kernel/src/pm/thread/interrupted.rs`, no exec/spec/proof equivalent)
  **Status:** Not fixed.
  **Description:** `join_cond()` remains omitted; the spec explicitly states Condvar is out-of-scope. This leaves join synchronization behavior outside the verification boundary and the module is still incomplete relative to the original API.
  **Suggested Fix:** Provide a minimal boundary model for `Condvar` and a `join_cond()` spec (or `external_body`) that at least ties the return value to the underlying `ThreadState` identity.

### Low
- **Location:** `ReadyThread` boundary model (exec/spec in `interrupted.rs` / `interrupted.spec.rs`)
  **Status:** Not fixed (documented).
  **Description:** The boundary model still omits `admission_time` from the real `ReadyThread::from_state`. Documentation now explains this is out-of-scope, but the model remains non-equivalent for scheduling/timing properties.
  **Suggested Fix:** If scheduling correctness is in scope, extend the model or verify `ReadyThread` itself and compose the guarantees here.

## Positive Observations
- The trust-boundary documentation is clearer and explicitly lists omitted APIs and assumptions.
- The core resume transition properties remain well specified and proven.

## Summary
The changes are mostly documentation; the core verification gaps remain. `thread_state_mut` is still an unverified escape hatch, `join_cond` is still missing, and the ReadyThread boundary model still omits `admission_time`. Verification is therefore not fully sound or complete for the original API surface.
