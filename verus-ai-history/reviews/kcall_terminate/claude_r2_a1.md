# Review: kcall_terminate (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

_None._

### High

- **Location:** `process_manager_terminate` (exec: terminate.rs, lines 270–322)
- **Description:** The `external_body` postcondition for `process_manager_terminate` does not model the case where the target PID is in the `interrupted` or `zombie` state. In the real PM implementation (`mod.rs:1036–1078`), terminate only checks the `ready` and `suspended` queues; interrupted and zombie processes fall through to the "process not found" error (`NoSuchProcess`). The spec's `ProcessManagerStateView` uses a single flat `process_set: Set<nat>` that does not distinguish process states (ready, suspended, interrupted, zombie). This means the postcondition claiming success implies `spec_pm_has_process(pm_pre, pid)` is an over-approximation: a PID can be in `process_set` (as an interrupted or zombie process) yet the real PM would return `NoSuchProcess`. The `external_body` postconditions are technically consistent (they don't claim success for non-ready/non-suspended PIDs), but the abstract model cannot express or verify that interrupted/zombie PIDs are correctly rejected.
- **Suggested Fix:** Either (a) extend `ProcessManagerStateView` with per-state sets (e.g., `ready_set`, `suspended_set`, `interrupted_set`, `zombie_set`) and refine the postcondition to require the PID be in `ready_set ∪ suspended_set` for success, or (b) document this as an explicit abstraction gap in the Trust Boundaries section and add a proof obligation note for the PM module's own verification.

### Medium

- **Location:** `process_manager_terminate` postcondition (exec: terminate.rs, lines 301–306)
- **Description:** The frame condition for success (`ret.1@.process_set.subset_of(pm_pre.process_set)` and other-PIDs-unchanged) is correct but incomplete for the real PM behavior. In the real code, a ready process with runnable threads is terminated and then _resumed back to the ready queue_ — so the PID is NOT removed. A suspended process is moved to the interrupted queue. The postcondition correctly avoids claiming PID removal (`!spec_pm_has_process(post, pid)`), which is documented. However, it also does not assert PID _preservation_ on the success path for the resume case (i.e., `spec_pm_has_process(ret.1@, pid)` when the process had runnable threads). This means the success postcondition is weaker than the actual behavior — it allows PID removal even though it never happens in the resume case.
- **Suggested Fix:** This is acceptable as a conservative under-specification, but consider adding a comment noting that the success-path frame could be tightened to `spec_pm_has_process(ret.1@, pid)` for the resume sub-case if per-state modeling is added.

- **Location:** `terminate_model` (exec: terminate.rs, line 413)
- **Description:** In the PID parse error branch, the ghost terminate outcome is set to `TerminateOutcomeView::TmOk` (a dummy value since the PM was never called). While this is functionally harmless (the spec short-circuits on PidError so the terminate outcome is irrelevant), using `TmOk` as a sentinel for "never executed" is semantically misleading. A reader might incorrectly infer that the terminate step succeeded.
- **Suggested Fix:** Either add a comment explaining the dummy value choice, or introduce a `TmNotCalled` variant to `TerminateOutcomeView` for clarity. Alternatively, use `TmError { error_code: 0 }` as a more obviously invalid sentinel. Low priority since `lemma_pid_parse_short_circuit` proves the value is irrelevant.

- **Location:** `spec_is_valid_pid` (spec: terminate.spec.rs, line 176)
- **Description:** `spec_is_valid_pid` is declared as `uninterp spec fn` (uninterpreted). This means the verifier knows nothing about which raw u32 values are valid PIDs. While the `try_from_process_identifier` external_body ties the parse result to this predicate (determinism), no concrete properties of PID validity are available in this module. For example, the verifier cannot prove that `spec_is_valid_pid(0)` is true (PID 0 should be a valid PID — it's the kernel PID). This means the postcondition `arg0 as nat == KERNEL_PID() && spec_pid_parsed_ok(ret.1@) ==> spec_is_error(ret.0.spec_view())` (line 391) has the antecedent `spec_pid_parsed_ok(ret.1@)` which cannot be established from `arg0 == 0` alone.
- **Suggested Fix:** Add an axiom or a lemma (e.g., `axiom_kernel_pid_is_valid`) that `spec_is_valid_pid(KERNEL_PID())` is true. This would let the verifier prove that terminate(0) always fails unconditionally (not just conditionally on successful parse). Alternatively, if the pid module's verification establishes this, reference it.

### Low

- **Location:** `terminate_model` return type (exec: terminate.rs, line 357)
- **Description:** The function returns a 4-tuple `(KcallResultModel, Ghost<PidParseOutcomeView>, Ghost<TerminateOutcomeView>, Ghost<ProcessManagerStateView>)`. This is a wide return type that makes postconditions harder to read. Consider using a named struct for clarity.
- **Suggested Fix:** Define a `TerminateModelResult` struct with named fields. Low priority; the current approach works.

- **Location:** Documentation (exec: terminate.rs, lines 66–105)
- **Description:** The module-level documentation is exceptionally thorough, listing all verified properties, out-of-scope properties, trust boundaries, logging rationale, and an API mapping table. Minor nit: the "API Mapping" table lists `pub fn terminate(pm, args) -> KcallResult` but the verified model is `terminate_model(arg0, Ghost(pm_pre))` — the parameter abstraction (dropping `pm: &mut ProcessManager` in favor of `Ghost<ProcessManagerStateView>` and dropping `args: &KcallArgs` in favor of `arg0: u32`) could be explicitly noted as an intentional simplification.
- **Suggested Fix:** Add a sentence to the API Mapping notes column explaining the parameter abstraction.

- **Location:** `lemma_success_requires_terminatable` (proof: terminate.proof.rs, lines 408–419)
- **Description:** This lemma's `requires` clause duplicates the body of `spec_terminate_possible`, and its `ensures` just asserts `spec_terminate_possible(pm_pre, pid)`. The proof is trivially true by unfolding the spec function. While it serves as documentation that the postconditions of `terminate_model` imply `spec_terminate_possible`, it could be made more useful by taking the actual exec postconditions as preconditions rather than pre-decomposed predicates.
- **Suggested Fix:** Restructure the lemma to take `spec_is_success(result)` and the relevant postconditions as requires, then prove `spec_terminate_possible`. This would make the lemma a genuine composition proof rather than a tautology.

## Positive Observations

- **Excellent documentation.** The module-level comment block (lines 4–105) is among the best I've seen in verified systems code. It clearly enumerates verified properties, out-of-scope properties, trust boundaries with justifications, logging rationale, and an API mapping table.
- **Clean spec/proof/exec separation.** The three-file split is well-organized: specs define the abstract model, proofs establish lemmas over the spec, and exec code threads ghost state through the pipeline. The `include!` mechanism keeps them logically connected.
- **No `assume` statements.** The core module contains zero `assume` invocations. All trusted assumptions are confined to `external_body` functions with explicit postconditions.
- **Comprehensive error path coverage.** All three error paths (PID parse failure, kernel PID, non-existent PID) are modeled with error code preservation lemmas. The running-PID rejection is also covered.
- **State preservation on error is proven.** The `lemma_state_unchanged_on_error` proof covers both error paths in a single lemma, showing PM state immutability on any failure.
- **Well-formedness invariant.** The `spec_pm_wf` predicate preventing inconsistent postconditions (running + non-existent) is a thoughtful addition that demonstrates understanding of the PM's structural invariants.
- **Verification passes cleanly.** All 19 verification conditions pass with zero errors.
- **Conservative trust boundaries.** The `external_body` contracts are carefully scoped — notably avoiding the claim that terminate removes the PID, which would be incorrect for the resume case.
- **Mutual exclusion and exhaustiveness.** `lemma_result_exhaustive` proves both exhaustiveness and mutual exclusion of result variants, a good completeness property.

## Summary

This is a high-quality verification of the `kcall_terminate` dispatch function. The spec cleanly models the two-step pipeline (PID parse → PM terminate), and the proof suite covers error propagation, error code preservation, state preservation, kernel/running PID protection, and result exhaustiveness. The trust boundaries are conservative and well-documented.

The primary gap is the flat `process_set` abstraction in `ProcessManagerStateView`, which cannot distinguish process lifecycle states (ready, suspended, interrupted, zombie). This means the model cannot express or verify that interrupted/zombie processes are correctly rejected by terminate. This is an inherent limitation of modeling the PM as a single set — it's not incorrect (the external_body postconditions don't claim anything false), but it represents an abstraction gap where real PM behavior is under-specified.

The secondary observation is that `spec_is_valid_pid` being uninterpreted means the module cannot unconditionally prove that terminating PID 0 always fails — it can only prove it fails _if PID 0 parses successfully_. Adding a single axiom linking KERNEL_PID to valid PID space would close this gap.

Overall, the verification demonstrates strong methodology: careful trust boundary identification, conservative postconditions, thorough lemma coverage, and excellent documentation. The grade reflects the high quality with a minor deduction for the process-state abstraction gap and the uninterpreted PID validity predicate.
