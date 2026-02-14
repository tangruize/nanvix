# Review: running_thread Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Public method specs expose `self.state` (impl field) instead of `self@.state` (view field).**
  Per the guidelines (Step 3): "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`."
  Affected ensures clauses (in `running.rs`):
  - `sleep()` line 294: `result.state@ == self.state@` — should use `result@.state == self@.state`.
  - `schedule()` line 320: `result.state@ == self.state@` — same issue.
  - `exit()` line 381: `result.state@ == self.state@` — same issue.
  - `thread_state()` line 352: `result@ == self.state@` — should be `result@ == self@.state`.
  - `from_state()` line 264: `result.state@ == state@` — exposes `result.state` field of the output `RunningThread`.

  These postconditions leak the internal field name `state` from `RunningThread`, `SleepingThread`, `ReadyThread`, and `ZombieThread` into public contracts. The methodology requires using view types (`self@.state`, `result@.state`) exclusively in public specs.

- **`id()` and `thread_state()` lack `requires self.wf()` precondition.**
  Per the guidelines (Step 3): "for any input `Self` parameters, one of the preconditions should be that `inv` holds." Both `id()` (line 336) and `thread_state()` (line 349) are public methods taking `&self` but do not require `self.wf()`. While they may be trivially correct without it, the methodology mandates the invariant precondition on all public methods for consistency.

### Medium
- **`view()` is `open spec fn` instead of `pub closed spec fn`.**
  The spec file notes at line 245–248 explain this is forced by Verus's `View` trait. This is a justified deviation and well-documented. No action needed, but it represents a methodology deviation that should be tracked.

- **Some `pub open spec fn` accessors in spec file expose internal structure.**
  Spec functions like `spec_id()`, `spec_locked_mutex_count()`, `spec_is_interrupted()` (lines 75–127 of running.spec.rs) are `pub open` and delegate to `self.state.spec_*()`, which exposes the `state` field. The methodology recommends only `view()` and `wf()`/`inv()` be public spec functions on the impl type (Step 3: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`"). These accessors would be more appropriate on `RunningThreadView`.

- **Proof lemmas (e.g., `lemma_new_is_drop_safe`) access raw impl fields like `r.state.locked_mutex_count` and `r.state.locked_mutex_set@`.**
  Lines 328–330 of running.proof.rs reference internal fields in `requires` clauses of `pub proof fn` lemmas. While proofs inherently need internal access, these are public lemmas whose preconditions leak implementation details. Consider making these private or wrapping the preconditions in spec helpers.

### Low
- **Trigger warnings from Verus.** Multiple `forall` quantifiers auto-chose triggers with low confidence (verification output). Consider adding explicit `#[trigger]` annotations or `#![auto]` to suppress warnings and improve SMT solver predictability.

- **`thread_state_mut()` is `#[verifier::external]`.** This is well-documented with clear trust obligations (lines 476–506). The justification (Verus cannot express `&mut T` return types) is valid. No action needed, but it is an unverified escape hatch.

## Summary

The running_thread module demonstrates strong spec methodology adherence overall. Verification passes cleanly (47 verified, 0 errors) with no `assume`, `admit`, or unjustified `external_body` annotations. View types correctly use abstract types (`int`, `Option<int>`, `ThreadStateView`). The `wf()` function is correctly `pub closed spec fn`, and all state-transitioning public methods (`sleep`, `schedule`, `exit`, `put_mutex_guard`, `take_mutex_guard`) properly require/ensure `wf()`.

The two high-priority issues are: (1) public method postconditions reference `self.state` / `result.state` directly instead of going through the view (`self@.state` / `result@.state`), which leaks implementation field names; and (2) `id()` and `thread_state()` omit the `requires self.wf()` precondition mandated by the methodology. Additionally, the existence of multiple `pub open spec fn` accessors beyond `view()` and `wf()` on the impl type is a minor methodology deviation — the guidelines suggest these belong on the View type instead.

The `#[verifier::external]` on `thread_state_mut()` is the only unverified escape hatch, and it is thoroughly documented with explicit trust obligations. The boundary models for `SleepingThread`, `ReadyThread`, and `ZombieThread` are well-structured with clear cross-module verification obligations documented.
