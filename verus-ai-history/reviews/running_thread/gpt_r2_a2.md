# Review: running_thread (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage gap persists: `join_cond()` still omitted** (exec/spec: `verus/split/kernel/pm/thread/running.rs`, `running.spec.rs`).
  - **Evidence:** The updated files explicitly state that `Condvar` is elided and `join_cond()` is omitted, but there is still no modeled or stubbed `join_cond()` API. See `running.rs` module docs noting omission and trust boundary (`running.rs` lines ~35-44) and the spec’s trust assumptions repeating the omission (`running.spec.rs` lines ~17-25).
  - **Why this matters:** The interface is still not fully covered, leaving a public API unverified. This is a direct carry‑over from the previous review and was not fixed—only documented.
  - **Status:** **Not fixed**.

- **API strengthening remains for mutex guards** (exec/spec: `verus/split/kernel/pm/thread/running.rs`).
  - **Evidence:** `put_mutex_guard` still requires `!spec_has_mutex(address@)` and `take_mutex_guard` still requires `spec_has_mutex(address@)` (see `running.rs` lines ~401–440). The runtime API returns `Option<MutexGuard>` and allows overwrite/missing cases, but the spec still eliminates those paths. A new comment labels this as “API Strengthening Note” (lines ~425–430), but the behavior is still stronger than the implementation.
  - **Why this matters:** This is still an unproven trust assumption (T2), not a verification of the actual API. Unless all call sites are proven to satisfy these preconditions, this is a soundness gap.
  - **Status:** **Not fixed** (only documented).

- **`thread_state_mut()` remains an unchecked external escape hatch** (exec: `verus/split/kernel/pm/thread/running.rs`).
  - **Evidence:** `thread_state_mut()` is still `#[verifier::external]` with no machine-checked postconditions; only comments state intended obligations (`running.rs` lines ~463–483).
  - **Why this matters:** This allows arbitrary mutation of `ThreadState` without verifying preservation of `wf()` or identity. The comment does not close the soundness hole.
  - **Status:** **Not fixed** (only documented).

### Low
- None.

## Positive Observations
- The updated files are explicit about the trust boundaries and assumptions, which improves traceability of gaps.
- The documentation now clearly labels the API strengthening for mutex guards, making the assumption visible to reviewers.

## Summary
The prover did not actually fix the previously reported issues; they remain as explicit omissions or trust assumptions. `join_cond()` is still missing, the mutex-guard APIs are still strengthened, and `thread_state_mut()` remains an unchecked external escape hatch. Verification is therefore **not complete** and still relies on unproven assumptions.
