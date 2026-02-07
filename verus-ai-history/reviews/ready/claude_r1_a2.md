# Review: ready — Round 2 (claude-opus-4.6)

## Grade: A

## Previous Issues — Disposition

### Medium: `thread_state_mut()` unverified mutation surface

**Status: FIXED (substantively).**

The prover added three verified forwarding methods inside the `verus!` block:
- `set_interrupt_reason(&mut self, reason: int)` (line 330)
- `store_mutex_guard(&mut self, address: Ghost<int>)` (line 354)
- `take_mutex_guard(&mut self, address: Ghost<int>)` (line 380)

Each forwards to the corresponding `ThreadState` method with machine-checked pre/postconditions including identity preservation, well-formedness, mutex accounting, and admission time stability. The `thread_state_mut()` documentation (lines 472–474) now directs callers to prefer these verified alternatives. The `#[verifier::external]` escape hatch is retained only for opaque HAL operations (`fpu_state_mut()`, `context_mut()`), which is the correct residual trust boundary.

**Verification:** All three methods pass Verus verification (34 total obligations, 0 errors). The postconditions correctly compose with the underlying `ThreadState` method specs.

**Minor residual observation (not blocking):** The forwarding methods drop some frame conditions that the underlying `ThreadState` methods guarantee. For example, `ReadyThread::set_interrupt_reason` does not expose `spec_kernel_stack`, `spec_user_stack`, or `spec_user_tda` preservation, while `ThreadState::set_interrupt_reason` does. Similarly, `store_mutex_guard` and `take_mutex_guard` don't expose `spec_interrupt_reason` preservation. This is acceptable because the ReadyThread specs are transparent pass-throughs — a caller who needs these frame conditions can open the spec definitions — but it means the forwarding methods are strictly weaker than calling `ThreadState` methods directly. A future caller who needs to prove, e.g., that `set_interrupt_reason` doesn't change their user stack, would need to reason through the spec definitions rather than relying solely on the postcondition.

### Medium: `EXIT_STATUS_INTERRUPTED()` hardcoded to 0

**Status: FIXED (correctly).**

Changed from `0` to `4` (line 70 of spec file), with a comment citing the source: `src/libs/sysapi/src/errno.rs:21`. Independently verified: `ErrorCode::Interrupted = EINTR` (`#[repr(i32)]` enum), `EINTR: c_int = 4`, `From<ErrorCode> for i32` casts via `errno as i32`. The value `4` is faithful to the runtime.

### Low: `clock_now()` no postcondition

**Status: FIXED (correctly).**

Added `ensures result >= 0` (line 77 of exec file), with updated documentation (lines 69–73) explaining the postcondition. The `SystemTime` type represents non-negative durations, so `>= 0` is a sound minimal postcondition.

**Minor residual (not blocking):** The `new()` and `from_state()` postconditions do not expose `result.spec_admission_time() >= 0`, even though the `clock_now()` postcondition makes this provable. Callers cannot establish admission time non-negativity from the constructor specs alone without opening the implementation.

### Low: `join_cond()` omitted entirely

**Status: REJECTED — Rejection is justified.**

The prover's reasoning is correct: `join_cond` is a field on `ThreadState` in the original source (`src/kernel/src/pm/thread/state.rs:56`), and `ReadyThread::join_cond()` simply calls `self.state.join_cond()`. Modeling it would require adding `Condvar` to the verified `ThreadState` model, which is a dependency-level change outside the scope of a `ReadyThread` review. The omission is already documented in the module-level comments (line 29 of exec file, line 24 of spec file).

### Low: ReadyThread struct fields are pub

**Status: ACKNOWLEDGED — No action needed.**

No change made, consistent with the original review's recommendation. This is standard Verus practice.

### Low: Proof lemmas have empty bodies

**Status: FIXED (partially).**

A composite lemma `lemma_new_then_run` was added (proof file lines 252–285) that proves construction followed by interrupt-clearing preserves identity, well-formedness, and drop safety. The lemma is SMT-discharged (empty body), which confirms the spec composition is consistent.

**Observation:** The lemma constructs `ThreadState` and `ReadyThread` directly at the proof level rather than invoking the exec `new()` and `run()` functions. It proves properties about spec-level struct construction with `interrupt_reason: None` applied twice (initial state already has `None`, then "clearing" to `None` is a no-op). This means it exercises the algebraic composition of spec definitions rather than the exec function postcondition chain. It is still a valid regression guard, but a stronger version would reference the `run()` postconditions (e.g., `result.interrupt_reason == self.spec_interrupt_reason()`) to catch regressions in the exec specs.

## New Issues Introduced

### Low

- **Location:** `store_mutex_guard` precondition (exec, line 357)
  **Description:** The precondition `old(self).state.locked_mutex_count < usize::MAX` references the internal `state` field directly instead of using the public spec function `spec_locked_mutex_count()`. This is an abstraction leak: callers must know about the `state` field to satisfy this precondition, even though all other preconditions/postconditions use spec functions. The underlying `ThreadState::store_mutex_guard` has the same pattern (it uses `old(self).locked_mutex_count`), but at the ReadyThread level this crosses the encapsulation boundary.
  **Suggested Fix:** Replace with `old(self).spec_locked_mutex_count() < usize::MAX as nat` (adjusting types as needed), or document this as a Verus modeling artifact if the `nat`/`usize` type mismatch makes the spec-level formulation awkward.

- **Location:** `clock_now()` documentation (spec file line 22, exec file line 42–44)
  **Description:** Two documentation locations still say `clock_now()` has "no spec-level constraint," which is now stale after adding `ensures result >= 0`.
  **Suggested Fix:** Update spec file line 22 to: `clock_now()` is `external_body`: minimal postcondition `result >= 0`. Update exec file trust boundary section similarly.

## Positive Observations

- **Genuine reduction of trust surface:** The three forwarding methods cover the common mutation paths (interrupt management, mutex accounting) with full machine-checked specs. The `#[verifier::external]` escape hatch now has a clearly documented, narrow residual purpose (opaque HAL access only). This is exactly the mitigation the original review recommended.
- **Faithful exit status constant:** `EXIT_STATUS_INTERRUPTED() = 4` is independently traceable through the source: `ErrorCode::Interrupted = EINTR`, `EINTR = 4`, `From<ErrorCode> for i32` casts the discriminant. The spec comment cites the exact source location.
- **Admission time stability:** All three forwarding methods include `self.spec_admission_time() == old(self).spec_admission_time()` as a postcondition, which is a good frame condition that isn't present on the underlying `ThreadState` methods (since ThreadState doesn't know about admission time). This shows the prover thought about ReadyThread-specific invariants beyond just forwarding.
- **Verification count increased:** 30 → 34 verified obligations with 0 errors, confirming that the new methods and lemma are genuinely checked by the SMT solver.
- **All previous positives retained:** Complete function coverage, zero `assume` statements, strong state transition specs, boundary model pattern, mutex accounting, drop safety propagation, clean three-file separation.

## Summary

The prover addressed the previous review's issues substantively and correctly. The `EXIT_STATUS_INTERRUPTED` fix from `0` to `4` is faithful to the source. The `clock_now()` postcondition is sound. The verified forwarding methods genuinely reduce the unverified mutation surface of `thread_state_mut()`. The `join_cond()` rejection is justified with evidence.

Two minor documentation inconsistencies remain (stale "no constraint" text for `clock_now()`), and one abstraction leak in `store_mutex_guard`'s precondition. Neither affects verification soundness. The module is well-verified with 34 obligations passing cleanly.
