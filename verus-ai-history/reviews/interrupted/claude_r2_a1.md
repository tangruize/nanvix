# Review: interrupted (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

- **Location:** `thread_state_mut()` in exec (`interrupted.rs:231`)
  - **Description:** `thread_state_mut` is marked `#[verifier::external]`, bypassing all verification. This function returns `&mut ThreadState`, allowing arbitrary mutation of the underlying state without any machine-checked guarantees. A caller could change the thread ID, corrupt mutex accounting, or violate `wf()`. While this is due to a genuine Verus limitation (`&mut T` return types are unsupported), the function exists in the original source and is part of the public API. Any caller of `thread_state_mut` operates entirely outside the verification boundary, and there are no runtime assertions to guard the invariants.
  - **Suggested Fix:** Document all call sites of `thread_state_mut` in the trust boundary section. Consider adding debug assertions (`debug_assert!`) inside the function or at call sites to check `wf()` and identity preservation at runtime when Verus cannot verify them. Alternatively, refactor callers to use specific setter methods (e.g., `set_user_tda`, `set_kernel_stack`) that can be individually verified with postconditions.

### Medium

- **Location:** `join_cond()` — omitted from verified code
  - **Description:** The original `InterruptedThread::join_cond()` is a public function that is omitted entirely from the verified code. The omission is documented and justified (opaque `Condvar` type), but this means the verification has incomplete function coverage. While the function is a simple passthrough with no state mutation, it is part of the public API and its absence means the verified module is not a complete model.
  - **Suggested Fix:** Add an `#[verifier::external_body]` stub for `join_cond()` with a comment marking it as trust boundary, rather than omitting it entirely. This would make the coverage gap explicit in the verified code rather than only in documentation.

- **Location:** `ReadyThread` boundary model in exec (`interrupted.rs:90-93`)
  - **Description:** The `ReadyThread` boundary model omits the `admission_time` field present in the real `ReadyThread`. While this is documented and the omission is reasonable (scheduling property), there is no cross-module check that the boundary model's postconditions are actually implied by the real `ReadyThread::from_state`. If the real `ReadyThread` is verified separately with different invariants, the boundary model here could become stale or inconsistent.
  - **Suggested Fix:** Add a comment or tracking issue requiring that when `ReadyThread` is independently verified, the postconditions of this boundary model (`spec_id`, `spec_interrupt_reason`, `wf`) are confirmed as implied by the real spec. Consider adding a cross-module consistency lemma or test.

- **Location:** `set_interrupt_reason` in `state.rs` (verified) vs original (`state.rs:204`)
  - **Description:** The verified `set_interrupt_reason` accepts any `int` without restricting it to a valid `InterruptReason` variant. In the original code, the parameter type is `InterruptReason` (an enum), which statically enforces that only `Killed` or `TimedOut` can be passed. The verification relies on `InterruptedThread::wf()` to ensure the valid-reason constraint, but `set_interrupt_reason` is a `pub` function on `ThreadState` that could theoretically be called from other modules with an invalid int. The constraint is correctly enforced at the `InterruptedThread::resume()` call site, but the `ThreadState` API is more permissive than the original.
  - **Suggested Fix:** Add a `spec_valid_reason` precondition to `ThreadState::set_interrupt_reason`, or document that the widened type is intentional and that validity is enforced at the `InterruptedThread` layer. This is acceptable as-is because the only verified call path (`resume`) does enforce validity.

### Low

- **Location:** `InterruptedThread` struct fields are `pub` in exec (`interrupted.rs:68-73`)
  - **Description:** The original `InterruptedThread` has private fields (`state` and `reason`), but the verified version makes them `pub` for Verus spec access. The code includes a comment explaining this, but it weakens the encapsulation model — external code could construct an `InterruptedThread` directly without going through `from_state`, bypassing `wf()` establishment.
  - **Suggested Fix:** This is a known Verus limitation. The documentation is adequate. No code change needed, but consider adding a note that `wf()` must be established by callers if constructing directly.

- **Location:** Proof file — `lemma_resume_*` lemmas manually reconstruct post-state (`interrupted.proof.rs:92-95`)
  - **Description:** The resume lemmas construct the post-state via struct update syntax (`ThreadState { interrupt_reason: Some(self.reason), ..self.state }`). This is fragile: if the exec `resume` implementation changes to modify additional fields, these lemmas could silently become vacuous or fail to capture the change. The code includes a NOTE comment about this.
  - **Suggested Fix:** The NOTE comment on line 78-81 of the proof file is good practice. No additional change needed, but consider adding a meta-lemma or assertion in `resume` that the post-state matches the proof's reconstruction.

- **Location:** `InterruptReason` modeled as `int` rather than a Verus `enum`
  - **Description:** The original `InterruptReason` is a Rust enum with two variants. The verified version models it as an `int` with a validity predicate `spec_valid_reason`. While this is a reasonable abstraction, Verus does support enum types, and using a proper enum would provide exhaustiveness checking automatically rather than requiring manual `lemma_valid_reason_exhaustive`.
  - **Suggested Fix:** Consider modeling `InterruptReason` as a Verus enum in future iterations. This is cosmetic and the current `int` + validity predicate approach is sound.

## Positive Observations

- **Verification passes cleanly:** All 21 verification conditions pass with no errors.
- **Excellent documentation:** The trust boundary, verification model, and trust assumptions are thoroughly documented in all three files. The omission of `join_cond()` and the `&mut T` limitation are clearly explained.
- **Clean spec/proof/exec separation:** The three-file split is well-organized with specs defining the abstract model, proofs establishing invariant properties, and exec implementing the verified logic.
- **Key safety property proven:** The critical property — that `resume()` correctly stamps the interrupt reason onto `ThreadState` before transitioning to `ReadyThread` — is proven with machine-checked postconditions (`result.spec_interrupt_reason() == Some(self.spec_reason())`).
- **Comprehensive frame conditions:** The postconditions on `resume()` prove identity preservation, well-formedness preservation, and correct interrupt reason propagation.
- **Rich proof library:** The proof file contains useful lemmas beyond what's strictly needed for the exec code (e.g., reason distinctness, view equality, mutex preservation through resume), providing a solid foundation for downstream verification.
- **Correct semantic equivalence:** The verified `resume()` faithfully models the original: `self.state.set_interrupt_reason(self.reason)` followed by `ReadyThread::from_state(self.state)`, matching the original control flow exactly.

## Summary

The verification of `InterruptedThread` is solid and well-executed. The core state-transition safety property (interrupt reason propagation through `resume()`) is fully machine-checked. The verification model makes reasonable abstractions (`Box` → direct, `InterruptReason` → `int`, `Condvar` → omitted) that are all clearly documented.

The main gaps are: (1) `thread_state_mut` being `#[verifier::external]` due to a Verus limitation, which creates an unverified escape hatch for state mutation, and (2) the omission of `join_cond()`, which reduces function coverage. Both are well-documented trust boundary items rather than verification bugs.

The `ReadyThread` boundary model is a pragmatic approach for cross-module verification, but requires manual tracking to ensure consistency with the real `ReadyThread` spec when that module is verified independently.

Overall, this is a high-quality verification effort that captures the essential correctness properties of the interrupted thread state transition protocol with appropriate trust boundary documentation.
