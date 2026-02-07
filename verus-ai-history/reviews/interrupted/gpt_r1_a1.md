# Review: interrupted (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedThread::thread_state_mut` (exec; `verus/split/kernel/pm/thread/interrupted.rs`)
  - **Description:** The function is outside the `verus!` block and has no specification or proof obligations. It returns `&mut ThreadState`, allowing arbitrary mutation of the state (including `id`, interrupt reason, and mutex accounting) without preserving `wf()` or identity. This is a soundness gap and also breaks the coverage requirement for a public API.
  - **Suggested Fix:** Replace the raw `&mut` accessor with verified setter methods that carry pre/postconditions, or move the API behind an `external_body` with explicit documented postconditions. Once Verus supports `&mut` returns, add full specs ensuring `wf()` and identity preservation.

### Medium
- **Location:** `InterruptedThread::join_cond` (missing; original `src/kernel/src/pm/thread/interrupted.rs`)
  - **Description:** The public `join_cond()` accessor is omitted entirely from the verified model. This fails the coverage requirement and leaves join-related synchronization behavior unmodeled.
  - **Suggested Fix:** Add a boundary spec for `join_cond()` (e.g., model `Condvar` as an opaque token with identity/equality properties) or mark it as `external_body` with a minimal, explicit specification to preserve equivalence.

- **Location:** `ReadyThread::from_state` boundary model (exec/spec; `verus/split/kernel/pm/thread/interrupted.rs`)
  - **Description:** The verified `ReadyThread` omits `admission_time`, but the real `ReadyThread::from_state` sets `admission_time = clock::now()`. This side effect is not captured, so the resume transition is not fully equivalent to the original behavior.
  - **Suggested Fix:** Extend the boundary model with an abstract `admission_time` field and specify that `from_state` updates it, or explicitly document and justify that admission-time semantics are intentionally out of scope.

### Low
- **Location:** `InterruptedThread::thread_state` (exec; `verus/split/kernel/pm/thread/interrupted.rs`)
  - **Description:** The spec only guarantees identity preservation (`spec_id`) for the returned reference. It does not state that the reference reflects the full underlying state view, making the accessor specification weaker than the original semantics.
  - **Suggested Fix:** Strengthen the postcondition to `result@ == self.state@` (or equivalent view equality) so consumers can rely on full-state transparency.

## Positive Observations
- `resume()` is specified to stamp the interrupt reason onto the state and preserve identity and well-formedness, matching the core safety requirement.
- `wf()` precisely captures state well-formedness plus valid interrupt reason, and the proof file includes dedicated lemmas for identity and reason preservation.
- The trust boundary and modeling choices (Box elision, reason tags) are clearly documented.

## Summary
The verification captures the core state-transition safety property in `resume()` and maintains identity and well-formedness, but it misses coverage for `join_cond()` and leaves `thread_state_mut()` as a significant unverified escape hatch. The ReadyThread boundary model also omits the admission-time update, which weakens equivalence. Tightening these areas would materially improve completeness and soundness.
