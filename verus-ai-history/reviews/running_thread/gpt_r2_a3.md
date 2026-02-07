# Review: running_thread (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage gap persists: `join_cond()` still omitted** (exec/spec: `verus/split/kernel/pm/thread/running.rs`, `running.spec.rs`).
  - **Evidence:** Both exec and spec still explicitly state that `Condvar` and `join_cond()` are omitted (exec module docs around lines ~35–44; spec trust assumptions around lines ~17–25). No stub or modeled API is present.
  - **Why this matters:** The interface remains partially unverified. This was flagged previously and has not been fixed.
  - **Status:** **Not fixed**.

- **API strengthening remains for mutex guards** (exec: `verus/split/kernel/pm/thread/running.rs`).
  - **Evidence:** `put_mutex_guard` still requires `!spec_has_mutex(address@)` and `take_mutex_guard` still requires `spec_has_mutex(address@)` (see exec around lines ~401–440). This still excludes the original `Option<MutexGuard>` missing/overwrite paths.
  - **Why this matters:** This remains a trust assumption (T2) rather than a verification of the real API behavior. It is unchanged from the prior review.
  - **Status:** **Not fixed**.

- **`thread_state_mut()` remains an unchecked external escape hatch** (exec: `verus/split/kernel/pm/thread/running.rs`).
  - **Evidence:** The function is still `#[verifier::external]` with no machine-checked postconditions; only comments describe intended obligations (lines ~463–483).
  - **Why this matters:** It allows arbitrary mutation without verified preservation of `wf()` or identity. This remains a soundness hole if used in verified code paths.
  - **Status:** **Not fixed**.

### Low
- None.

## Positive Observations
- The trust boundary comments remain explicit, which at least makes the assumptions auditable.
- No new regressions were introduced in the spec/proof files.

## Summary
The prover did not materially address the previous review. The same three issues remain: `join_cond()` is omitted, mutex-guard APIs are still strengthened, and `thread_state_mut()` remains an unchecked external escape hatch. Verification is **not complete or sound** until these gaps are resolved.
