# Review: kcall_scoreboard (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

*None.*

### High

- **Location:** `dispatch()` in scoreboard.rs (exec), lines 769–843
  - **Description:** The `dispatch()` method inlines both the handler and dispatcher sides of the protocol into a single sequential function. This means the handler steps (`handle()` + `handled()`) are executed within the same function call as the dispatcher steps. While the documentation acknowledges this as a modeling choice for the full cycle, it collapses the trust boundary between the two threads: in the original, the dispatcher thread calls `dispatch()` and the kernel thread independently calls `handle()` and `handled()`. The verified `dispatch()` proves that *if* both sides cooperate in sequence, the state machine is correct — but it does not model the case where the handler misbehaves (e.g., calls `handled()` with the wrong phase, or calls `handle()` twice). The split API (`begin_dispatch` + `handle` + `handled` + `complete_dispatch`) does model individual transitions with preconditions, but the combined `dispatch()` bypasses these by directly mutating fields.
  - **Suggested Fix:** The combined `dispatch()` is acceptable as a convenience lemma/wrapper for the happy path, but the review should note that the *real* verification strength comes from the split API. Consider adding a proof lemma that composes the split API calls and shows equivalence to the monolithic `dispatch()`, rather than having `dispatch()` do raw field mutations. This would strengthen the connection between the two APIs.

- **Location:** `handle()` in scoreboard.rs (exec) vs original `ScoreBoard::handle(&self)` in mod.rs line 176
  - **Description:** The original `handle()` takes `&self` (immutable reference) and uses atomic `try_down()`, meaning it can be called concurrently by the handler thread without exclusive access. The verified model takes `&mut self`, which *assumes* mutual exclusion between `handle()` and `dispatch()`. The documentation at T4 acknowledges this, but this is a significant semantic divergence: the atomicity guarantee of `try_down()` is what ensures the handler and dispatcher don't race on the `dispatched` semaphore. The sequential model cannot capture this.
  - **Suggested Fix:** This is acknowledged in trust boundary T4 and is a known Verus limitation. No code change needed, but it should be clearly flagged as the primary gap in the verification. The T4 refinement argument (mutex + semaphore enforce total order) is sound for the happy path but doesn't cover the case where `try_down` is called from a non-handler thread.

### Medium

- **Location:** `KcallResult` model in scoreboard.rs (exec), lines 241–247
  - **Description:** The original `KcallResult` is an enum (`Success(KcallSuccess)` / `Error(KcallError)`) and derives `Copy`. The verified model uses `is_success: bool` + `value: i64`, which is semantically equivalent for the scoreboard protocol. However, the `wf()` spec (line 205–207 in spec) constrains error values to i32 range, but there is no enforcement at the exec level — `KcallResult` can be constructed with arbitrary `is_success`/`value` combinations via direct field access (fields are `pub`). The `error()` constructor enforces i32 input, but the `handled()` method only requires `ret.wf()` as a precondition, meaning a caller could pass a non-wf result if they constructed it directly.
  - **Suggested Fix:** This is mitigated by the `ret.wf()` precondition on `handled()` and `dispatch()`, which Verus will check at all call sites. The pub fields are documented as a Verus necessity. Low risk in practice, but could add an assertion `assert(ret.wf())` in `handled()` body for defense in depth.

- **Location:** `abandon_dispatch()` in scoreboard.rs (exec), lines 622–636
  - **Description:** The `abandon_dispatch()` only models one specific recovery failure mode: the mutex guard dropping. In the original, if `handled.down()` is interrupted via `SleepError::Interrupted`, the `_guard: MutexGuard` goes out of scope, unlocking the mutex. However, the handler thread may have already consumed the `dispatched` signal and be in the middle of processing (Dispatched phase) or may have already signaled `handled` (Handled phase). The verified model's `dispatch()` only models abandonment from the Signaled phase (before handler runs), which is the "most conservative" per the docs. The split `abandon_dispatch()` accepts any non-Idle phase, which is more general. The gap is that the monolithic `dispatch()` doesn't demonstrate abandonment from Dispatched or Handled phases.
  - **Suggested Fix:** Add proof lemmas showing `abandon_dispatch` composed from each active phase (Dispatched, Handled) produces a characterized stuck state, similar to `lemma_dispatch_interrupted_stuck` but for the other phases. The individual `abandon_dispatch()` exec function already handles this, but explicit proofs from each phase would strengthen coverage.

- **Location:** `ScoreBoardSlot::get_board()` in scoreboard.rs (exec), lines 1016–1025
  - **Description:** Returns `&ScoreBoard` (immutable reference), but the original `get_mut()` returns `&'static mut ScoreBoard`. The verified model cannot perform mutable operations through this reference — callers must use `slot.board` directly. This is documented but means the init → get_mut → dispatch workflow of the original isn't directly testable through the slot API.
  - **Suggested Fix:** Documented limitation. Consider adding a `get_board_mut(&mut self)` that returns `&mut ScoreBoard` and preserves slot well-formedness, to provide a closer API mapping.

### Low

- **Location:** Module-level `pub fn init()` in mod.rs (original), line 192–195
  - **Description:** The top-level `init()` function (which logs and calls `ScoreBoard::init()`) is listed as out of scope. This is reasonable since it's a trivial logging wrapper.
  - **Suggested Fix:** None needed.

- **Location:** `impl Debug for KcallArgs` in mod.rs (original), lines 65–74
  - **Description:** Not modeled, which is appropriate since formatting has no safety implications.
  - **Suggested Fix:** None needed.

- **Location:** `completed_cycles` ghost field in scoreboard.rs (exec), line 295
  - **Description:** This is verification-only ghost state (`Ghost<nat>`) with no original counterpart. While useful for inductive proofs, it introduces a concept that doesn't exist in the original. The proofs about cycle counting (e.g., `lemma_n_cycles_count`) verify a property of the *model*, not the original code.
  - **Suggested Fix:** This is standard practice in Verus verification. The ghost field is erased at runtime. No change needed, but note that cycle-counter properties are model-level, not code-level.

- **Location:** `spec_n_identical_cycles` in scoreboard.spec.rs, lines 481–495
  - **Description:** Uses identical args/ret for all cycles. The comment notes this but the generalization to varying inputs per cycle is only argued informally ("each cycle increments by 1 regardless of args/ret"). A formal lemma for heterogeneous cycles would be stronger.
  - **Suggested Fix:** Consider adding a `spec_n_heterogeneous_cycles` using a sequence of (args, ret) pairs, or a lemma proving the cycle counter property holds for arbitrary per-cycle inputs. Low priority since the single-cycle property (`lemma_cycle_counter_monotonic`) already establishes the key fact.

## Positive Observations

- **Excellent documentation:** The module-level doc comment (lines 1–188 of scoreboard.rs) is outstanding. The API Mapping table, Trust Boundaries (T1–T5), API Divergence section, and Verification Scope section provide thorough transparency about what is and isn't verified. This is among the best-documented Verus verification modules I've reviewed.

- **No assume/external_body/trusted:** The entire verification (65 verified, 0 errors) is achieved without any `assume`, `external_body`, or `trusted` annotations. All proofs are fully mechanized.

- **Comprehensive state machine coverage:** The four-phase protocol (Idle → Signaled → Dispatched → Handled → Idle) is fully specified with individual transition specs (`spec_begin_dispatch`, `spec_handle`, `spec_handled`, `spec_complete_dispatch`) and a composed full-cycle spec. The well-formedness invariant (`wf()`) ties together phase, mutex, and semaphore state consistently.

- **Error path modeling:** Both success and error paths are modeled: lock failure (`try_begin_dispatch`), semaphore try_down failure (`try_handle`), and `handled.down()` interruption (`abandon_dispatch`). The proofs that error paths preserve state or produce characterized stuck states are valuable.

- **Semaphore overflow proofs:** `lemma_semaphore_up_dispatched_cannot_fail` and `lemma_semaphore_up_handled_cannot_fail` prove that `up()` is called only when the semaphore value is 0, ruling out overflow. This is a non-obvious property that connects the protocol phase to semaphore correctness.

- **Clean spec/proof/exec separation:** The three-file split (scoreboard.rs / scoreboard.spec.rs / scoreboard.proof.rs) cleanly separates concerns. Spec functions define the state machine, proof lemmas verify properties, and exec code implements the transitions with pre/post conditions linking to specs.

- **Injectivity proof:** `lemma_different_inputs_different_outputs` proves that different arguments/results produce observably different outcomes, ruling out information loss in the protocol.

- **Data integrity proofs:** `lemma_result_integrity` and `lemma_args_integrity` prove that the handler reads exactly what the dispatcher set, and the dispatcher reads exactly what the handler returned. These are the core correctness properties of the rendezvous channel.

## Summary

This is a high-quality Verus verification of the kernel call scoreboard protocol. The state machine model faithfully captures the four-phase handshake (Idle → Signaled → Dispatched → Handled → Idle) with explicit semaphore signal/consume tracking, mutex modeling, and comprehensive error path coverage. All 65 verification conditions pass without any `assume` or `external_body` escape hatches.

The primary limitation is the sequential model's inability to verify the concurrent aspects of the protocol (the `&self` → `&mut self` change for `handle()` and the lack of atomicity reasoning). This is a known Verus limitation, and the documentation provides a sound refinement argument for why the sequential model is a valid abstraction. The trust boundaries (T1–T5) are honestly documented.

The verification would be strengthened by: (1) composing the split API into the monolithic `dispatch()` via proof rather than re-implementing with raw field mutations, (2) adding explicit abandonment proofs from each active phase, and (3) providing a `get_board_mut` on the slot for closer API fidelity. These are medium-priority improvements that would elevate the grade to A/A+.

Overall, this verification provides strong evidence of sequential state machine correctness, data integrity, and error path safety for the scoreboard protocol. The documentation quality is exemplary and sets a high standard for verification transparency.
