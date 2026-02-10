# Review: kcall_handler (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

*(none)*

### High

- **Location:** `run_iteration()` (exec, handler.rs:434–475)
  - **Description:** `run_iteration()` computes `should_yield` but never calls `yield_cpu()`. The original handler loop calls `ProcessManager::giveup()` when no work is done. The verified model returns a flag but never actually performs the yield, meaning the yield *behavior* is unverified. Similarly, the function never calls `poll_scoreboard()` — it receives the poll result as a parameter, which means the scoreboard polling is also outside the verified boundary.
  - **Suggested Fix:** Either (a) have `run_iteration()` call `yield_cpu()` when `should_yield` is true and `poll_scoreboard()` internally, or (b) introduce a higher-level `run_iteration_full()` that composes polling, iteration, and yield, with ensures clauses covering the complete behavior.

- **Location:** `drain_remaining_zombies()` (exec, handler.rs:496–500)
  - **Description:** This `external_body` function has **no postcondition**. The original post-loop drain is a simple `while let` that harvests all remaining zombies. This is a meaningful behavioral property (cleanup completeness) that goes entirely unverified.
  - **Suggested Fix:** Add an ensures clause, e.g., modeling that after return, no zombies remain. Even a weaker postcondition like `ensures true` would be more honest than the current silent contract.

### Medium

- **Location:** `poll_messages()` (exec, handler.rs:224–228)
  - **Description:** This `external_body` returns `bool` with **no postcondition**. The original IKC polling loop has meaningful logic: it checks `number_buffered_messages < MAX_IKC_MESSAGES`, iterates up to `IKC_POLL_BATCH_SIZE` times, and posts messages to destinations. None of this is captured, even abstractly.
  - **Suggested Fix:** At minimum, document why no postcondition is provided. Ideally, add a postcondition relating the return value to whether any messages were actually processed.

- **Location:** `harvest_zombies()` (exec, handler.rs:238–242)
  - **Description:** This `external_body` returns `ZombieHarvestResult` with **no postcondition**. There is no constraint ensuring that `is_initd` is true iff `pid == SPEC_INITD_PID()`. The spec function `spec_should_terminate` relies on `is_initd` but nothing ties it to the actual PID value in the exec model.
  - **Suggested Fix:** Add `ensures result.is_initd <==> result.pid == 1u32` to link the INITD flag to the PID. This would close the gap between the abstract termination spec and the concrete exec model.

- **Location:** `handle_kcall_phase()` ensures clause (exec, handler.rs:384–386)
  - **Description:** The postcondition only guarantees `was_invalid_syscall` for kcall numbers 1 and 2. It does **not** guarantee that numbers classified as `Invalid` by the spec (3, 5, 9, 20, 22–27, 29, 32+) also produce `was_invalid_syscall == true`. The `classify_and_check_invalid` function handles these correctly at exec level, but the phase-level ensures clause doesn't propagate this.
  - **Suggested Fix:** Strengthen ensures to: `poll.has_call && spec_returns_invalid_syscall(poll.kcall_number) ==> result.was_invalid_syscall`.

- **Location:** Loop structure (exec, handler.rs overall)
  - **Description:** The original `kcall_handler` is a loop with `break` on INITD termination. The verification only models a **single iteration** via `run_iteration()`. There is no loop invariant, no inductive proof that the loop correctly sequences iterations, and no proof that the loop terminates only when INITD exits. The spec functions `spec_should_terminate` etc. describe single-step behavior but don't compose into a loop correctness argument.
  - **Suggested Fix:** Add a ghost loop model or inductive lemma showing that: (1) the loop invariant holds at each iteration boundary, (2) the only exit path is INITD termination, and (3) the returned exit status matches INITD's status.

- **Location:** `run_iteration()` ensures clause (exec, handler.rs:436–443)
  - **Description:** No postcondition links `should_terminate` to `exit_status`. When `should_terminate` is true, the `exit_status` field carries the INITD exit status, but nothing in the ensures clause guarantees this. The caller has no verified contract about what `exit_status` means.
  - **Suggested Fix:** Add: `result.should_terminate ==> result.exit_status == harvest.exit_status` where harvest is the INITD zombie result. Alternatively, specify that `should_terminate` implies the exit status originated from the INITD process.

### Low

- **Location:** `spec_iteration_wf()` and `spec_dispatch_category_wf()` (spec, handler.spec.rs:343–354)
  - **Description:** Both well-formedness predicates unconditionally return `true`. They serve no verification purpose and add no constraints. They are documented as "included for uniformity" but provide a false sense of specification coverage.
  - **Suggested Fix:** Either give them meaningful bodies (e.g., constrain `LoopIterationState` fields to be consistent) or remove them to reduce spec noise.

- **Location:** `lemma_no_pending_no_kcall()` (proof, handler.proof.rs:349–356)
  - **Description:** This "lemma" requires `!state.kcall_handled` and ensures `!state.kcall_handled` — it's a tautology. It proves nothing that isn't already given in the precondition.
  - **Suggested Fix:** Either strengthen the lemma to prove something about the state after a NoPending poll outcome (e.g., that the state is unchanged), or remove it.

- **Location:** Error handling paths not modeled (exec, handler.rs)
  - **Description:** The original has several error-and-continue paths: `scoreboard.handled(ret)` can fail (warn and continue), `harvest_zombies` can fail (error and continue), `ProcessManager::giveup()` can fail (error and continue). The verified model silently succeeds on all these paths. This is acceptable for a control-flow model but should be documented as a scope limitation.
  - **Suggested Fix:** Add a note in the doc header's "Scope Limitations" section about error recovery paths being modeled as always-succeed.

- **Location:** `dispatch_to_subsystem()` postcondition (exec, handler.rs:211)
  - **Description:** The postcondition uses implication (`==>`) for kcall numbers 1 and 2, but `classify_and_check_invalid()` already handles these by constructing `make_invalid_syscall_error()` directly — `dispatch_to_subsystem()` is never called for numbers 1 or 2. The postcondition on the external_body is thus vacuously true for all actual invocations.
  - **Suggested Fix:** Remove the vacuous postcondition or add postconditions that constrain actual dispatch behavior for valid kcall numbers.

## Positive Observations

- **Excellent documentation**: The module header (handler.rs lines 1–99) is outstanding — it clearly lists verified properties, the API mapping table, trust boundaries, and scope limitations. This is a model for verification documentation.
- **All 31 verification conditions pass** with no errors.
- **Dispatch classification is thoroughly verified**: The spec `spec_classify_handler_kcall` correctly maps all 32 kcall numbers (0–31) to their handler categories, matching the source `KcallNumber` enum exactly. The proof lemmas cover totality, correctness, invalid classification, and partition properties.
- **No `assume` statements** in any of the three files — all proofs are honest.
- **Kcall number values are correct**: Cross-referencing with `src/libs/sys/src/sys/number.rs`, all 21 handler kcall numbers and their mappings to `HandlerDispatchCategory` variants are accurate.
- **Clean spec/proof/exec separation**: Spec functions define abstract behavior, proof lemmas establish properties, and exec functions implement the model. The `include!` pattern keeps files focused.
- **Work flag monotonicity proofs** (proof lines 240–287) are a nice touch — they prove that once work is recorded, it can't be un-recorded within an iteration.
- **Trust boundaries are clearly identified** (T1–T5) with explicit rationale for each.

## Summary

The verification provides solid coverage of the kcall dispatch routing and per-iteration work tracking logic. The dispatch classification is thoroughly verified with 21 correct number-to-category mappings, and the yield/idle logic is well-proven. The termination condition (INITD-only exit) is clearly specified.

The main gaps are: (1) the loop structure itself is unverified — only single iterations are modeled, with no inductive proof of loop correctness; (2) several external_body functions (`harvest_zombies`, `poll_messages`, `drain_remaining_zombies`) lack postconditions, creating trust gaps; (3) `yield_cpu()` is declared but never called in the verified model, leaving the yield behavior unverified; and (4) the ensures clause on `run_iteration()` doesn't link termination to exit status. These are addressable without fundamental restructuring. Recommended next steps: add loop invariant reasoning, strengthen external_body postconditions (especially `harvest_zombies` linking `is_initd` to PID), and wire `yield_cpu()` into the iteration model.
