# Review: interrupted_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- Location: InterruptedProcess::state/state_mut (exec, interrupted.rs)
  - Description: Both accessors are marked `#[verifier::external_body]`, so their postconditions are assumed rather than proven. These are core APIs and can be implemented as pure ghost returns, so the current use introduces unnecessary trusted code and violates the "no unjustified external_body" criterion.
  - Suggested Fix: Replace `external_body` with executable ghost implementations (e.g., return `Ghost(self.pid@)` for `state` and for `state_mut` return `Ghost(self.pid@)` without mutating fields) and keep the same ensures.

### Medium
- Location: InterruptedProcess::find_thread/find_thread_mut (exec, interrupted.rs) and spec_find_thread (spec, interrupted.spec.rs)
  - Description: The verified exec bodies are spec-only wrappers returning a ghost `Option<int>` and do not relate to the real iterator-based search implementation. This leaves the actual search logic (predicate and list order) unverified, so semantic equivalence to the original is not established.
  - Suggested Fix: Model search over ghost sequences with an executable Verus function (if supported) or introduce a trusted refinement lemma that connects the real implementation to `spec_find_thread` with explicit assumptions and documented scope.
- Location: InterruptedProcess::resume (exec/spec, interrupted.rs) vs ReadyThread::from_state (src/kernel/src/pm/thread/ready.rs)
  - Description: The model fixes `ready_admission_times` to `[0]` and only asserts non-negativity, while the real code sets admission time via `clock::now()`. This loses the semantic link to the current time and weakens scheduling-related correctness properties.
  - Suggested Fix: Extend the model to carry a ghost time parameter or a spec for `clock::now()` and assert that the admission time equals that value; alternatively, make the admission time an unconstrained ghost with an explicit relation to a time model.

### Low
- Location: interrupt (exec/spec, interrupted.rs)
  - Description: The model only preserves thread identity and elides the `InterruptReason::Killed` semantics from the original implementation.
  - Suggested Fix: Add a ghost field or spec predicate capturing the interrupt reason (or a dedicated postcondition stating the reason is `Killed`) if downstream proofs rely on it.

## Positive Observations
- All original functions have corresponding verified versions with clear specs.
- `resume()` correctly models front-pop semantics and preserves PID and other thread lists while establishing RunnableProcess well-formedness.
- Invariants for non-emptiness, disjointness, and uniqueness are consistently enforced and preserved.

## Summary
The verification captures the core state-machine transitions and structural invariants, but there are notable trust gaps: the accessor methods are external-body and the thread search is spec-only, leaving real behavior unverified. Strengthening these areas and modeling admission times more faithfully would improve soundness and semantic equivalence.
