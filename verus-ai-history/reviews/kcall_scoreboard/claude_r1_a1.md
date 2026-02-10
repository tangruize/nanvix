# Review: kcall_scoreboard (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

(none)

### High

- **H1: Semaphore signaling not actually modeled**
  - Priority: High
  - Location: `begin_dispatch()` (exec, line 319–324); `spec_dispatched_count()` (spec, line 229–231)
  - Description: In the original `dispatch()`, `self.dispatched.up()` increments the semaphore to 1, and `handle()` calls `self.dispatched.try_down()` which decrements it back to 0. In the model, `begin_dispatch()` sets `dispatched_value = 0` directly, and `handle()` is a no-op on state. The transient `dispatched_value == 1` state is never modeled, so the actual signaling mechanism between dispatcher and handler is elided. The comment says "we keep it at 0 since the handler will consume it immediately" but this assumption conflates two separate thread operations into one, bypassing the very synchronization the scoreboard exists to provide. `spec_dispatched_count()` always returns 0, making the `dispatched_value` field and its invariant constraints meaningless.
  - Suggested Fix: Model `begin_dispatch()` as setting `dispatched_value = 1`, and `handle()` as consuming it (setting it to 0). This properly models the signal/wait handshake even in the sequential model, and makes the `dispatched_value` field meaningful in the invariant.

- **H2: `KcallResult::wf()` is vacuously true**
  - Priority: High
  - Location: `KcallResult::wf()` (spec, line 180–182)
  - Description: The spec `i64::MIN as i64 <= self.value && self.value <= i64::MAX as i64` is always true for an `i64` field. This means the `wf()` precondition on `ret` in `handled()` provides no constraint, and `self.result.wf()` in `ScoreBoard::wf()` is vacuously satisfied. Any i64 value passes.
  - Suggested Fix: Either remove the field from `wf()` (since it adds nothing), or model the original `KcallResult` enum semantics more faithfully — e.g., success values must be non-negative `KcallSuccess(i64)` where value >= 0, and error values must be valid `ErrorCode` discriminants (negative i32 range). This would give the well-formedness check actual teeth.

### Medium

- **M1: `lemma_n_cycles_count` is a tautology**
  - Priority: Medium
  - Location: `lemma_n_cycles_count()` (proof, line 243–249)
  - Description: The ensures clause says `initial_cycles + n == initial_cycles + n`, which is trivially true by reflexivity. The lemma name and doc comment suggest it should prove that after `n` full cycles starting from cycle count `c`, the scoreboard's `completed_cycles` field equals `c + n`. But the actual proof references no ScoreBoard state at all.
  - Suggested Fix: Define a recursive spec function `spec_n_cycles(view, n, args_seq, ret_seq)` that composes `n` full cycles, and prove `spec_n_cycles(view, n, ...).completed_cycles == view.completed_cycles + n` along with the Idle phase invariant.

- **M2: `lemma_transitions_deterministic` proves a triviality**
  - Priority: Medium
  - Location: `lemma_transitions_deterministic()` (proof, line 257–273)
  - Description: The preconditions require `args1 == args2` and `ret1 == ret2`, then the lemma proves that identical inputs produce identical outputs. This is trivially true for any pure function. The name "deterministic" suggests it should prove that the *same* state with the *same* input always yields the *same* output (i.e., the function is deterministic), but that's already guaranteed by Verus spec functions being pure.
  - Suggested Fix: Either remove this lemma (it adds no value) or replace it with a more meaningful property, such as proving that different args/results lead to observably different states (injectivity).

- **M3: `lemma_mutex_held_during_active_phases` only covers Dispatched**
  - Priority: Medium
  - Location: `lemma_mutex_held_during_active_phases()` (proof, line 204–213)
  - Description: The lemma name says "active phases" (plural), implying both Dispatched and Handled. But the ensures clause only proves `dispatched.locked` (i.e., the lock is held after `begin_dispatch`). It does not prove that the lock is also held in the Handled phase. The `lemma_handled_transition` does prove `after.locked`, but there is no combined lemma that demonstrates the lock is held throughout the entire Dispatched→Handled span.
  - Suggested Fix: Extend the ensures clause to also show that after `spec_handled(dispatched, ret)`, the lock is still held. Alternatively, rename to `lemma_mutex_held_after_dispatch`.

- **M4: Error paths not modeled**
  - Priority: Medium
  - Location: Entire exec module
  - Description: The original `dispatch()` returns `Result<KcallResult, SleepError>` and can fail at three points: `self.lock.lock(None)`, `self.dispatched.up()`, and `self.handled.down()`. `handle()` returns `Result<&KcallArgs, Error>` and can fail at `self.dispatched.try_down()`. `handled()` returns `Result<(), Error>` and can fail at `self.handled.up()`. The verified model uses preconditions to guarantee success, which means all error propagation logic is unverified. While trust boundary T5 documents this, the error paths in a kernel component are critical for system stability.
  - Suggested Fix: Consider modeling at least the "scoreboard uninitialized" error from `get_mut()` and the semaphore failure modes, even if simplified. Alternatively, explicitly document which error conditions are safety-critical vs. which are only relevant for liveness.

- **M5: `KcallArgs::wf()` restricts pid/tid to non-negative, original allows full i32 range**
  - Priority: Medium
  - Location: `KcallArgs::wf()` (spec, line 145–153)
  - Description: The spec requires `0 <= self.pid` and `0 <= self.tid`, but the original `ProcessIdentifier` and `ThreadIdentifier` are `From<i32>` wrappers that accept any `i32`. The default initialization uses `i32::MAX` (positive), but the restriction to non-negative values is an unverified assumption about the domain. If negative identifiers are ever used (e.g., sentinel values), this would be overly restrictive.
  - Suggested Fix: Either verify that the kernel never produces negative process/thread identifiers (trace through the ProcessIdentifier API), or relax the constraint to the full `i32` range.

### Low

- **L1: `completed_cycles` field has no original counterpart**
  - Priority: Low
  - Location: `ScoreBoard` struct (exec, line 157); all cycle-related lemmas
  - Description: The `completed_cycles` field is added purely for verification and does not exist in the original `ScoreBoard`. While this is a standard ghost-state technique, it introduces a semantic gap between the model and the original. The field also has a `u64::MAX` overflow guard in `complete_dispatch()` precondition that doesn't correspond to any original behavior.
  - Suggested Fix: Consider making `completed_cycles` a `Ghost<nat>` to make it explicit that this is verification-only state, and document in the API mapping table that this is an added ghost field.

- **L2: `Debug::fmt` not modeled**
  - Priority: Low
  - Location: Original `impl Debug for KcallArgs` (mod.rs, line 66–74)
  - Description: The `Debug` formatting implementation is not modeled. This is reasonable since it's display-only and has no state effects.
  - Suggested Fix: None needed. Acceptable omission.

- **L3: Standalone `pub fn init()` not modeled**
  - Priority: Low
  - Location: Original `pub fn init()` (mod.rs, line 192–195)
  - Description: The top-level `init()` function that calls `ScoreBoard::init()` is not modeled. It's a thin wrapper with only a log call.
  - Suggested Fix: None needed. The important logic is in `ScoreBoard::init()` / `new()`.

- **L4: `handle()` returns `Ghost` instead of reference**
  - Priority: Low
  - Location: `handle()` (exec, line 342–351)
  - Description: The original `handle()` returns `Result<&KcallArgs, Error>` — a reference to the args. The model returns `Ghost<KcallArgsView>`. While the ghost value preserves the spec-level data flow, a real exec function returning only a ghost value is not executable. This limits the model's faithfulness for downstream consumers that need to actually read the args.
  - Suggested Fix: Return `&KcallArgs` with an ensures clause that `result@ == self.args@`. The ghost return is fine for proof purposes but reduces the model's utility as a drop-in replacement.

## Positive Observations

- **Clean separation of concerns.** The spec/proof/exec split is well-executed. Each file has a clear role, and the `include!` composition is idiomatic for Verus.
- **Excellent documentation.** The module-level documentation in `scoreboard.rs` is thorough, with a clear API mapping table, trust boundary enumeration, and scope statement. This is exemplary for verified code.
- **Well-formedness invariant.** `ScoreBoard::wf()` ties together phase, lock state, and semaphore values coherently. The phase-consistency implications are complete for the modeled states.
- **Protocol correctness proven.** The three-phase handshake (Idle→Dispatched→Handled→Idle) is properly modeled and verified. The `spec_full_cycle` composition and `lemma_full_cycle_returns_to_idle` provide a good end-to-end correctness proof.
- **Data integrity properties.** `lemma_result_integrity` and `lemma_args_integrity` prove that data flows correctly through the scoreboard without corruption — the core safety property for a rendezvous channel.
- **No cheating.** Zero `assume`, `external_body`, or `trusted` annotations in any of the three files. All 32 verification conditions pass cleanly.
- **Trust boundaries are explicit.** The five trust boundaries (T1–T5) are clearly documented with rationale, enabling targeted future verification efforts.

## Summary

The verification provides a solid sequential state-machine model of the kcall scoreboard protocol. It correctly captures the three-phase handshake, data integrity, mutual exclusion, and cycle counting properties. The code is clean, well-documented, and free of unsound assumptions.

The main weaknesses are: (1) the semaphore signaling mechanism is modeled as a no-op rather than as an actual signal/consume pair, which undermines the claim of verifying the dispatch protocol; (2) several proof lemmas are tautological and don't prove meaningful properties; and (3) error paths are entirely unmodeled, which is significant for a kernel component.

The grade of B+ reflects strong verification infrastructure and documentation, correct high-level protocol modeling, and verified data integrity — offset by the signaling model gap, vacuous well-formedness checks, and weak lemmas that inflate the proof count without adding assurance.
