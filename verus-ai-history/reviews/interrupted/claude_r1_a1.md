# Review: interrupted (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `InterruptedThread::thread_state_mut` (exec, line 198)
  **Description:** `thread_state_mut` is placed outside the `verus!` block with no postconditions or invariant preservation guarantees. In the original code, callers can mutate internal thread state through this mutable reference. Without a specification, any caller could silently break the well-formedness invariant (`wf()`) or mutate the thread identifier. This is a verification gap — any mutation through this handle is unverified.
  **Suggested Fix:** Even though Verus does not support `&mut T` returns, add a comment-level trust annotation documenting the assumption that callers preserve `wf()` and `spec_id()`. Alternatively, once Verus supports mutable reference returns, add postconditions ensuring `self.wf()` holds on drop of the reference, or refactor to use specific setter methods instead.

### Medium

- **Location:** `InterruptedThread::resume` (exec, line 170)
  **Description:** The original `resume` takes `mut self` and calls `self.state.set_interrupt_reason(self.reason)` where `self.reason` is an `InterruptReason` enum value. The verified version uses `int` for the reason and calls `state.set_interrupt_reason(reason_value)` where `reason_value` is an `int`. The `set_interrupt_reason` on `ThreadState` has no precondition constraining the reason value to be valid. This means the verified `set_interrupt_reason` accepts any `int`, while the original only accepts `InterruptReason` enum variants. The well-formedness of `InterruptedThread` constrains this, but `set_interrupt_reason` itself is not guarded. This is a minor model gap — the constraint is enforced at a higher level but not at the `ThreadState` API boundary.
  **Suggested Fix:** Consider adding a precondition to `ThreadState::set_interrupt_reason` that requires the reason to be a valid variant (0 or 1), or document the trust assumption that only `InterruptedThread::resume` calls this method with validated reasons.

- **Location:** `join_cond()` — omitted (exec)
  **Description:** The original `InterruptedThread::join_cond()` is documented as omitted because `Condvar` is opaque. This is a reasonable boundary decision, but it means the verification cannot reason about the synchronization correctness of thread join operations involving interrupted threads.
  **Suggested Fix:** No code change needed. Document this as an explicit trust boundary item, which is already done in the module header. Consider adding a note in the spec file as well.

- **Location:** `InterruptedThread` struct fields (exec, line 63–68)
  **Description:** The struct fields `state` and `reason` are declared `pub` in the verified code, while in the original they are private (`state: Box<ThreadState>`, `reason: InterruptReason`). This allows external code to construct or modify `InterruptedThread` values bypassing `from_state`, potentially violating the well-formedness invariant. This weakens encapsulation compared to the original.
  **Suggested Fix:** If Verus supports it, mark fields as `pub(crate)` or private, restricting construction to `from_state`. If Verus requires `pub` fields for spec access, document that external direct construction is not intended and rely on the module boundary for enforcement.

### Low

- **Location:** `ReadyThread` boundary model (exec, lines 75–104)
  **Description:** The `ReadyThread` boundary model only models `from_state`. If the real `ReadyThread` has additional invariants beyond `state.wf()` (e.g., constraints from its own module), these are not captured here. The boundary model is sound for verifying `interrupted.rs` in isolation but may not compose correctly with the full `ReadyThread` verification.
  **Suggested Fix:** When the `ReadyThread` module is verified, cross-check that this boundary model's postconditions are implied by the real `ReadyThread::from_state` spec. Add a comment noting the cross-module verification dependency.

- **Location:** Proof lemmas (proof, lines 81–172)
  **Description:** Several proof lemmas (e.g., `lemma_resume_sets_reason`, `lemma_resume_preserves_id`, `lemma_resume_preserves_wf`) construct the post-state manually as `ThreadState { interrupt_reason: Some(self.reason), ..self.state }` rather than referencing the actual exec code's behavior. If the exec `resume` implementation were to change (e.g., modify additional fields), these lemmas would still pass but would be stale.
  **Suggested Fix:** Consider restructuring proofs to reference the exec function's postconditions rather than reconstructing the post-state independently. Alternatively, add a comment that these lemmas must be updated in sync with `resume`.

- **Location:** `InterruptedThreadView` / `ReadyThreadView` (spec, lines 40–52)
  **Description:** The view types are defined and wired up but are only used by the `lemma_view_equality` proof. They are not exercised in the exec postconditions, which use spec functions directly. The views add completeness but are not load-bearing in the current verification.
  **Suggested Fix:** No action needed — having view types is good practice for future composability. Optionally, use them in postconditions for consistency with Verus conventions.

## Positive Observations

- **Zero unsound assumptions:** No `assume`, `external_body`, or `trusted` markers anywhere in the module. All 21 verification conditions pass cleanly.
- **Strong state-transition specification:** The `resume` function's postconditions correctly capture the key safety property: the interrupt reason is stamped onto the `ThreadState` before transitioning to `ReadyThread`, with identity and well-formedness preserved.
- **Comprehensive proof coverage:** The proof file includes lemmas for construction, identity preservation, state transition correctness, mutex and stack frame preservation, reason variant distinctness, exhaustiveness, and view equality.
- **Clean separation:** Spec, proof, and exec code are well-separated into their respective files, following the split verification pattern consistently.
- **Good documentation:** The module header clearly documents the verification model, trust boundary, and what is/isn't verified. The rationale for omitting `join_cond()` and for using `int` instead of an enum is clearly stated.
- **Correct modeling of `Box`:** Abstracting `Box<ThreadState>` as `ThreadState` directly is sound — `Box` is a transparent allocation wrapper with no logical effect on the contained value.
- **Well-formedness invariant:** The compound `wf()` predicate correctly combines `state.wf()` with `spec_valid_reason(reason)`, ensuring both sub-components are valid.

## Summary

This is a solid verification of a relatively simple but safety-critical state-transition module. All six original public functions are accounted for: `from_state`, `id`, `thread_state`, `thread_state_mut`, `resume`, and `join_cond`. Five have full verified specifications; `thread_state_mut` is present but unspecified (Verus limitation), and `join_cond` is documented as out-of-scope (opaque sync type).

The primary concern is the unverified `thread_state_mut` — any caller mutating thread state through this handle operates outside the verification boundary. This is an inherent Verus limitation, not a modeling error, but it represents the most significant gap in the verification.

The modeling choices (enum→int, Box→direct, Condvar→omitted) are all well-justified and documented. The proof lemmas are comprehensive, covering not just the happy-path properties but also frame conditions (mutex, stack preservation) and variant exhaustiveness. The grade of A- reflects the high quality of the verification with a deduction for the `thread_state_mut` gap and the public field visibility issue.
