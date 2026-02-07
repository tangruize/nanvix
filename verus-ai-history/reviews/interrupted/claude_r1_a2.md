# Re-Review: interrupted (claude-opus-4.6, round 2)

## Grade: A

## Verification Status

Verification passes: 21 verified, 0 errors. No `assume`, `external_body`, or `trusted` markers in the module.

## Review of Previous Issue Resolutions

### Issue 1 (High): `thread_state_mut` unverified mutation gap

**Original:** `thread_state_mut` is outside the `verus!` block with no postconditions. Callers can silently break `wf()` or mutate the thread identifier.

**Prover's fix:** Added trust boundary documentation (interrupted.rs lines 196–220) specifying that callers MUST preserve `wf()` and `spec_id()`, and noting the Verus `&mut T` return limitation.

**Verdict: Fixed (documentation).** The trust boundary is now explicitly documented at the function site. This is the correct approach for a Verus limitation — the gap cannot be closed with current Verus capabilities, and documenting the caller obligation is the standard practice. The note about future refactoring once Verus supports `&mut T` returns is appropriate.

**Remaining risk:** The gap still exists at the verification level — callers that mutate through this handle are unverified. This is inherent to the Verus limitation and is acceptably documented.

### Issue 2 (Medium): `set_interrupt_reason` accepts any `int`

**Original:** `ThreadState::set_interrupt_reason` has no precondition constraining the reason to valid variants (0 or 1).

**Prover's fix:** Added trust assumption documentation in interrupted.spec.rs (lines 28–32) explaining that the constraint is enforced at the `InterruptedThread` level via `wf()`, and that only `resume` calls `set_interrupt_reason` with `self.reason` which is guaranteed valid by `self.wf()`.

**Verdict: Fixed (documentation).** I verified the claim:
- `resume` requires `self.wf()`, which requires `spec_valid_reason(self.reason)`.
- `resume` passes `self.reason` to `set_interrupt_reason`.
- Therefore, only valid reasons (0 or 1) flow into `set_interrupt_reason` from this module.
- `ThreadState` is a dependency module — adding a precondition there is out of scope for this module's verification.

The trust assumption is valid and correctly documented.

### Issue 3 (Medium): `join_cond()` omitted

**Original:** `join_cond()` is omitted because `Condvar` is opaque. Synchronization correctness is not verified.

**Prover's fix:** Added trust boundary note in interrupted.spec.rs (lines 25–27) documenting the omission and its scope.

**Verdict: Fixed (documentation).** The omission was already noted in the exec file header; it is now also documented in the spec file's Trust Assumptions section. This is appropriate — `Condvar` cannot be meaningfully modeled in a pure spec.

### Issue 4 (Medium): `pub` fields weaken encapsulation

**Original:** Struct fields are `pub` while the original code has them private, allowing bypass of `from_state`.

**Prover's fix:** Added documentation on the struct (interrupted.rs lines 63–67) explaining that Verus requires `pub` fields for `open spec fn` access and directing users to use `from_state`.

**Verdict: Fixed (documentation).** I verified that Verus does require `pub` fields for `open spec fn` definitions that access struct fields — the spec functions in interrupted.spec.rs (e.g., `spec_id`, `spec_reason`, `wf`) all access `self.state` and `self.reason` directly. The documentation correctly explains this Verus requirement.

### Issue 5 (Low): ReadyThread boundary model cross-check

**Original:** The `ReadyThread` boundary model may not compose with the full `ReadyThread` module's invariants.

**Prover's fix:** Added cross-module dependency comment (interrupted.rs lines 81–83) noting that the boundary model's postconditions must be confirmed as implied by the real `ReadyThread::from_state` spec.

**Verdict: Fixed (documentation).** The comment creates an actionable cross-verification obligation for when the `ReadyThread` module is independently verified.

### Issue 6 (Low): Proof lemma staleness risk

**Original:** Proof lemmas construct the post-state manually, which could become stale if `resume` changes.

**Prover's fix:** Added sync comment (interrupted.proof.rs lines 77–81) warning that these lemmas must be updated in sync with the exec `resume` implementation.

**Verdict: Fixed (documentation).** The comment clearly documents the maintenance obligation. Additionally, I note that the proof lemmas are supplementary — the exec `resume` function's postconditions are independently verified by Verus, so staleness in the lemmas would not introduce unsoundness; the lemmas would simply prove properties about a state that no longer matches the implementation.

### Issue 7 (Low): View types not load-bearing

**Original:** View types are defined but only used by `lemma_view_equality`, not by exec postconditions.

**Prover's response:** No action needed.

**Verdict: Correctly rejected.** The original review itself stated "No action needed." View types provide future composability and are good practice.

## New Issues

### None identified.

The prover's fixes are all documentation-only changes, which is appropriate because:
1. The exec/spec/proof logic was already correct (no bugs or logical errors).
2. The issues identified were about verification gaps inherent to Verus limitations (`&mut T`, `pub` fields) or boundary decisions (`Condvar` omission), not modeling errors.
3. Documentation is the correct response to boundary decisions and known-limitation gaps.

## Critical Verification of Exec Logic

I independently verified the `resume` function's correctness chain:
1. `resume` requires `self.wf()` → `self.state.wf() && spec_valid_reason(self.reason)`.
2. `set_interrupt_reason` requires `old(self).wf()` — satisfied by `self.state.wf()`.
3. `set_interrupt_reason` ensures `self.wf()`, `self.spec_interrupt_reason() == Some(reason)`, `self.spec_id() == old(self).spec_id()`.
4. `ReadyThread::from_state` requires `state.wf()` — satisfied by postcondition of step 3.
5. `ReadyThread::from_state` ensures `result.spec_id() == state.spec_id()`, `result.spec_interrupt_reason() == state.spec_interrupt_reason()`, `result.wf()`.
6. Chaining: `result.spec_id() == self.spec_id()` ✓, `result.spec_interrupt_reason() == Some(self.spec_reason())` ✓, `result.wf()` ✓.

The state transition is sound.

## Positive Observations

- **Zero unsound assumptions:** No `assume`, `external_body`, or `trusted` anywhere in the module.
- **Clear trust boundary documentation:** All verification gaps are now explicitly documented with rationale and future action items.
- **Correct modeling decisions:** `Box→direct`, `enum→int`, `Condvar→omitted` are all sound and well-justified.
- **Comprehensive proof coverage:** 13 proof lemmas covering construction, identity, state transition, frame conditions, variant properties, and view equality.
- **Clean separation:** Spec, proof, and exec are well-organized in their respective files.
- **Strong postconditions on `resume`:** The key safety property (interrupt reason correctly propagated to ThreadState during state transition) is fully verified.

## Summary

All 7 issues from the original review have been genuinely addressed. Six were fixed with appropriate documentation; one was correctly rejected as requiring no action. The fixes are documentation-only, which is the right approach — the original code had no logical errors, and the issues were about communicating verification boundaries and Verus limitations.

The module provides sound verification of the `InterruptedThread` state machine within its documented trust boundary. The grade improves from A- to A, reflecting the now-complete documentation of trust assumptions and cross-module dependencies.
