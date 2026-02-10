# Review: kcall_scoreboard (claude-opus-4.6)

## Grade: A-

## Previous Issues Disposition

### H1: Semaphore signaling not actually modeled → FIXED ✅
The prover completely reworked the protocol model. The three-phase state machine (Idle→Dispatched→Handled→Idle) has been replaced with a faithful four-phase model (Idle→Signaled→Dispatched→Handled→Idle). The `begin_dispatch()` now sets `dispatched_value = 1` (exec line 382), and `handle()` consumes it by setting `dispatched_value = 0` (exec line 418). The `spec_dispatched_count()` now correctly returns 1 for the Signaled phase and 0 otherwise (spec line 232–233). New lemmas `lemma_dispatched_signal_protocol` and `lemma_handled_signal_protocol` prove the end-to-end signal/consume pattern for both semaphores. This is a substantive and correct fix.

### H2: `KcallResult::wf()` is vacuously true → FIXED ✅
The `KcallResult` struct now includes an `is_success: bool` field (exec line 166), faithfully modeling the original `Success(KcallSuccess(i64))` / `Error(KcallError(i32))` enum. The `wf()` spec is now `self.is_success || (i32::MIN as i64 <= self.value && self.value <= i32::MAX as i64)` (spec line 192), which provides a meaningful constraint for error results while allowing any i64 for success — exactly matching the original types. New lemmas `lemma_success_always_wf` and `lemma_error_wf_constrains_range` prove the constraint's behavior. Verified against original: `KcallError` wraps `i32`, `KcallSuccess` wraps `i64` — the model is correct.

### M1: `lemma_n_cycles_count` is a tautology → FIXED ✅
A new recursive spec function `spec_n_identical_cycles` (spec lines 402–416) composes n full cycles. The lemma (proof lines 334–358) now proves inductively that after n cycles, `completed_cycles == view.completed_cycles + n` and the state returns to Idle with both semaphores at 0. The proof uses actual recursion with `decreases n` and invokes itself on the post-one-cycle state. This is a genuine inductive proof.

### M2: `lemma_transitions_deterministic` proves a triviality → FIXED ✅
Replaced with `lemma_different_inputs_different_outputs` (proof lines 368–388), which proves injectivity: different args produce different output args, and different results produce different output results. This follows from the full cycle preserving `args` and `ret` into the final state, but it's a meaningful property that the spec functions are injective w.r.t. inputs. The trivial old lemma was removed.

### M3: `lemma_mutex_held_during_active_phases` only covers Dispatched → FIXED ✅
The lemma (proof lines 283–302) now traces through all three active phases: `signaled.locked && dispatched.locked && handled.locked`. This is exactly what the original review requested.

### M4: Error paths not modeled → DOCUMENTED ✅
Trust boundary T5 (exec lines 96–105) now classifies error conditions by criticality:
- Safety-critical: None (all errors propagate, no UB).
- Liveness-critical: `SleepError::Interrupted` from `handled.down()`, `Error` from `handled.up()`.
- Non-critical: `ErrorCode::TryAgain` from polling `handle()`.

The classification is accurate — I verified against the original: `dispatch()` propagates errors via `?` without entering an unsafe/inconsistent state (the mutex guard ensures cleanup), and `handle()` failures just mean "no dispatch pending." The error paths remain unmodeled in code, but the documentation justifies this as acceptable for a sequential state machine verification.

### M5: `KcallArgs::wf()` restricts pid/tid to non-negative → FIXED ✅
`KcallArgs::wf()` now returns `true` (spec line 156–158), with documentation explaining that `ProcessIdentifier` and `ThreadIdentifier` accept any `i32` via `From<i32>`. I independently verified this: both types have `impl From<i32>` that directly converts without validation. The old non-negativity restriction was incorrect and has been properly removed.

### L1: `completed_cycles` field has no original counterpart → DOCUMENTED ✅
The API Divergence section (exec lines 76–79) now explicitly documents: "`completed_cycles: u64` is verification-only state. The original has no cycle counter. This field tracks protocol progress for inductive proofs. It has a `u64::MAX` overflow guard in `complete_dispatch()` that does not correspond to original behavior." The struct field comment (exec line 196) also marks it. The suggestion to use `Ghost<nat>` was not adopted, but the documentation is clear.

### L2: `Debug::fmt` not modeled → N/A (accepted as-is in R1)

### L3: Standalone `pub fn init()` not modeled → N/A (accepted as-is in R1)

### L4: `handle()` returns `Ghost` instead of reference → FIXED ✅
A new `get_args(&self)` method (exec lines 435–443) provides reference access to the args after `handle()` has consumed the signal. The API Divergence section (exec lines 70–73) explains the split: `handle()` requires `&mut self` (for signal consumption modeling) and returns `Ghost<KcallArgsView>`, while `get_args()` provides `&KcallArgs` reference access. This is a clean design that serves both verification and practical needs.

## New Issues Found

### Medium

- **M1-new: `KcallArgs::wf()` is vacuously true**
  - Priority: Medium
  - Location: `KcallArgs::wf()` (spec, line 156–158)
  - Description: `KcallArgs::wf()` now returns `true` unconditionally. While this correctly matches the original (which has no runtime validation on args), it means the `ensures result.wf()` in `KcallArgs::new()` (exec line 246) and any uses of `args.wf()` as a precondition are vacuous. The function exists but provides zero filtering power. This is a "dead spec" that might mislead readers into thinking some constraint is being checked. It would be cleaner to either (a) remove `wf()` entirely from `KcallArgs` since it adds nothing, or (b) add a meaningful constraint if one exists (e.g., `number` should be a valid syscall number range).
  - Suggested Fix: Remove `KcallArgs::wf()` and its uses since it's always true, or document inline that it's intentionally vacuous as a placeholder for future constraints.

### Low

- **L1-new: `spec_n_identical_cycles` limited to identical inputs**
  - Priority: Low
  - Location: `spec_n_identical_cycles` (spec, lines 402–416); `lemma_n_cycles_count` (proof, lines 334–358)
  - Description: The recursive spec and its inductive lemma only handle the case where all n cycles use the same `args` and `ret`. A more general `spec_n_cycles(view, args_seq, ret_seq, n)` taking sequences of per-cycle inputs would be strictly stronger. However, the current form is sufficient to prove the cycle counter property, since the counter increments by 1 regardless of the specific args/ret values in each cycle. The limitation is acceptable.
  - Suggested Fix: No code change needed. Could add a comment noting that the cycle count property generalizes to varying inputs.

- **L2-new: `completed_cycles` is exec `u64` not `Ghost<nat>`**
  - Priority: Low
  - Location: `ScoreBoard` struct (exec, line 197)
  - Description: The `completed_cycles` field is verification-only state but is an exec-level `u64`. Making it `Ghost<nat>` would: (a) make it explicit that it has no runtime cost, (b) avoid the artificial `u64::MAX` overflow guard in `complete_dispatch()` precondition (exec line 495), and (c) eliminate the semantic gap between the model and the original. The documentation correctly labels it as verification-only.
  - Suggested Fix: Consider changing to `Ghost<nat>` in a future revision. This is cosmetic and does not affect soundness.

## Positive Observations

- **Thorough issue resolution.** All 7 code-fixable issues from the previous review were genuinely addressed with substantive changes, not superficial patches. The prover reworked the state machine from 3 phases to 4, restructured `KcallResult` with a variant tag, and replaced trivial lemmas with meaningful ones.
- **Faithful semaphore modeling.** The signal/consume pattern is now explicitly tracked: `dispatched_value` transitions 0→1→0, `handled_value` transitions 0→1→0. The `spec_dispatched_count()` and `spec_handled_count()` functions connect semaphore values to phases. New dedicated lemmas prove the signal protocol.
- **Improved well-formedness.** `ScoreBoard::wf()` (spec lines 216–222) is now tighter and cleaner: it checks result well-formedness, semaphore consistency with phase, and mutex/phase agreement. The simplified form `self.phase != ScoreBoardPhase::Idle ==> self.locked` covers all active phases in one clause.
- **Excellent documentation improvements.** The API Divergence section (exec lines 68–79) is a standout addition, explicitly calling out every deviation from the original with justification. The trust boundary T5 error classification (safety/liveness/non-critical) provides genuine engineering insight.
- **40 verified conditions, zero assume/external_body.** Up from 32 in the previous version, with all new lemmas proving meaningful properties.
- **Clean inductive proof.** `lemma_n_cycles_count` with `spec_n_identical_cycles` is a proper inductive argument with recursion, base case, and a `decreases` clause.
- **Injectivity property.** `lemma_different_inputs_different_outputs` proves that the scoreboard faithfully preserves input identity — different dispatches produce observably different states.

## Summary

The prover has done an excellent job addressing all issues from the previous review. Every substantive issue was genuinely fixed with correct code changes:

- **H1** (semaphore modeling): Complete rework from a 3-phase to 4-phase state machine with explicit signal/consume transitions. This was the most significant fix.
- **H2** (vacuous wf): Restructured `KcallResult` with `is_success` variant tag and meaningful i32 range constraint for errors.
- **M1–M3**: Tautological lemmas replaced with real inductive proofs and complete coverage.
- **M4–M5, L1, L4**: Addressed through code fixes and thorough documentation.

The remaining issues are minor: a vacuous `KcallArgs::wf()` that correctly mirrors the original's lack of validation, and cosmetic concerns about the ghost counter representation. The verification is now a faithful sequential model of the scoreboard protocol, with sound proofs, no unsound assumptions, and comprehensive documentation of trust boundaries and API divergences.

The grade improvement from B+ to A- reflects the elimination of all high-priority issues and the substantial strengthening of both the model and its proofs.
