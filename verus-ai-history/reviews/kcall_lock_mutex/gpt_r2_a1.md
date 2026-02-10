# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_lock_mutex_safety_preconditions` (spec), `lock_mutex_model` (exec)
  - **Description:** The unsafe caller preconditions (not kernel process, no resources held, no PM reference) are modeled as uninterpreted predicates but never required by `lock_mutex_model` or discharged at call sites. This leaves the verified model silent about the safety contract that the original function documents, so the verification does not guarantee safe usage in kernel contexts.
  - **Suggested Fix:** Add these predicates as `requires` on `lock_mutex_model` or prove them in the PM/kcall dispatch verification and thread them into a wrapper spec that is explicitly verified at call sites.
- **Location:** `mutex_lock_model`/`put_mutex_guard_model` (exec)
  - **Description:** The guard token is a bare `Ghost<bool>` with no association to `mutex_addr`. This allows `put_mutex_guard_model` to accept any `guard_token@ == true` regardless of which mutex it came from, weakening the ownership invariant that the guard corresponds to the same mutex returned by `get_mutex_model`.
  - **Suggested Fix:** Strengthen the ghost token to include a mutex identity/address (e.g., `Ghost<MutexId>` or `Ghost<(bool, u32)>`) and require it to match the `mutex_addr` passed to `put_mutex_guard_model`.

### Low
- **Location:** `get_mutex_model`, `mutex_lock_model`, `put_mutex_guard_model` postconditions (exec)
  - **Description:** The comments claim error codes are valid `ErrorCode` discriminants, but the postconditions only assert `error_code != 0`. This is weaker than the original API, which constructs `Error` from a concrete `ErrorCode` enum, and could admit unreachable error values in the model.
  - **Suggested Fix:** Constrain error codes to `ErrorCode`’s valid range or model the enum explicitly in the external-body contracts.
- **Location:** `mutex_lock_model` (exec), overall spec
  - **Description:** No liveness/progress property is specified; `mutex_lock_model` may return any outcome (subject to the TimedOut/finite constraint), so the model does not prove that a finite timeout eventually produces `TimedOut` or that a lock becomes available. This leaves liveness aspects of mutex locking unverified.
  - **Suggested Fix:** Add progress assumptions or a separate liveness proof in the mutex module, then reflect those guarantees into the kcall-level spec if required.

## Positive Observations
- The spec precisely models the parsing of timeouts and the short-circuit pipeline, matching the original control flow.
- Error propagation is carefully categorized and proven with dedicated lemmas for each failure mode.
- The split between exec/spec/proof is clean, with clear documentation of trust boundaries and non-goals.

## Summary
The verification captures the core control-flow semantics and error propagation of `lock_mutex`, with solid specs and proofs for the pipeline. The main gaps are in modeling the unsafe caller preconditions and the guard ownership invariant, plus a lack of liveness guarantees. Strengthening these areas would make the verification closer to the OS-level correctness expectations.
