# Review: running_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `RunningThread::join_cond` (missing in verus `running.rs` exec/spec).
  **Description:** The original public method `join_cond()` is omitted from the verified model, so coverage is incomplete and no property is proven about returning the join condition variable or the associated liveness note (wake-all semantics). This breaks equivalence and leaves synchronization behavior unverified.
  **Suggested Fix:** Add a modeled `join_cond()` in `running.rs` with an opaque/ghost `Condvar` token and a spec that preserves identity and links to `ThreadState` (or add an `external_body` with explicit trust obligations), then prove it in `running.proof.rs`.

- **Location:** `RunningThread::store_mutex_guard` / `RunningThread::take_mutex_guard` (verus `running.rs` exec; relies on `ThreadState` spec in `state.rs`).
  **Description:** The verification strengthens the API: `store_mutex_guard` requires `!spec_has_mutex(address)` and `take_mutex_guard` requires `spec_has_mutex(address)` and returns unit, whereas the original allows duplicate insert/returns `Option<MutexGuard>` on absence. This removes the `None` path and silently assumes T1/T2 invariants, making the spec stronger than the real code.
  **Suggested Fix:** Model the `Option` return and permit the not-found case (with postconditions for both branches), or add a verified wrapper that enforces the preconditions at runtime and restricts callers to that wrapper.

### Medium
- **Location:** `RunningThread::thread_state_mut` (verus `running.rs` exec, `#[verifier::external]`).
  **Description:** The external method has no postconditions, so it can invalidate `wf()` and `spec_id()` without detection; this is a soundness hole in the core module. The comments describe intended obligations but they are not enforced.
  **Suggested Fix:** Provide a verified wrapper with explicit ensures for identity and well-formedness, or encode a trusted lemma/axiom that the caller must prove to use the mutable reference.

- **Location:** `RunningThread::sleep` / `schedule` / `exit` specs (verus `running.rs` exec).
  **Description:** Postconditions only preserve id/mutex/drop-safety; they do not assert that other `ThreadState` fields (kernel/user stacks, user_tda, interrupt_reason, etc.) are preserved. This makes the spec too weak to rule out unintended state mutation across transitions.
  **Suggested Fix:** Strengthen ensures to preserve the full `ThreadStateView` (e.g., `result.state@ == self.state@` or equality of all relevant spec fields), or add a lemma stating view preservation across transitions.

### Low
- **Location:** `RunningThread::sleep` / `schedule` / `exit` (verus `running.rs` exec).
  **Description:** The raw `*mut ContextInformation` return values are omitted, so there is no verified relation between the returned context pointer and the underlying `ThreadState`. This leaves a boundary where scheduler/CPU context correctness is not modeled.
  **Suggested Fix:** Introduce an abstract context token or ghost pointer identity, and specify that the returned token corresponds to the same state to support downstream proofs.

- **Location:** `ReadyThread` boundary model (verus `running.rs` exec).
  **Description:** The admission time set by `ReadyThread::from_state` in the original code is intentionally omitted, so scheduling fairness/time-ordering properties cannot be stated or proven in this module.
  **Suggested Fix:** Add an abstract `admission_time` field with a minimal spec (`>= 0`, monotonic or fresh) and thread it through `schedule()` if those properties are needed.

## Positive Observations
- The split between exec/spec/proof is clean and well-documented, with explicit trust boundary notes and cross-module obligations.
- Core safety properties (identity preservation, mutex accounting, drop safety, alarm/status capture) are proven with clear lemmas.
- The model aligns with the original structure (state wrapper with transitions), making future strengthening straightforward.

## Summary
The verification is solid for identity and mutex accounting, but it misses a public API (`join_cond`) and relies on strengthened mutex preconditions plus an external mutable-state escape hatch. Tightening the transition specs to preserve full `ThreadState` and modeling the omitted APIs/pointers would significantly improve equivalence and soundness. Overall the split is clean, but key boundaries remain unverified.
