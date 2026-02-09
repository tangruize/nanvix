# Review: interrupted_process (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- None.

### Medium

1. **Ghost-only (design-level) verification — no executable code verified**
   - Location: All functions in `interrupted.rs` (exec)
   - Description: Every struct field is `Ghost<...>` and all functions operate purely on ghost sequences. The exec-mode code constructs ghost values only — no real `VecDeque`, `Box<ProcessState>`, or thread types are ever manipulated. This means the verification proves the *state-transition logic* of an abstract model, but does NOT verify the actual Rust implementation in `src/kernel/src/pm/process/state/interrupted.rs`. Bugs in real container operations (e.g., `pop_front` returning the wrong element, `NonEmptyVecDeque::from` mishandling empty deques) would not be caught.
   - Suggested Fix: This is clearly documented in the file header and is a deliberate design choice. Long-term, `external_body` wrappers for `NonEmptyVecDeque`/`VecDeque` and exec-mode functions operating on real data would close this gap. No action needed now, but this should be tracked as future work.

2. **`find_thread`/`find_thread_mut` are spec-only stubs — executable search logic not verified**
   - Location: `find_thread()` and `find_thread_mut()` in `interrupted.rs` (exec), lines 498–530
   - Description: Both functions simply return `Ghost(self.spec_find_thread(tid@))`. The original performs `iter().find(|t| t.id() == tid)` across three collections in priority order (interrupted → sleeping → zombie). A bug in the real search predicate, collection ordering, or early-return logic would not be detected. The trust gap is documented in `lemma_find_thread_refinement_assumption` and `spec_find_thread_integration_obligation`, but remains unverified.
   - Suggested Fix: This is a Verus limitation (cannot express `Option<ThreadRef<'_>>` return types or iterate ghost sequences). The documentation and integration obligation specs are appropriate. If Verus later supports reference-typed returns, replace with a verified implementation.

3. **`resume_with_valid_clock` is an extra function not in the original source**
   - Location: `resume_with_valid_clock()` in `interrupted.rs` (exec), lines 445–468
   - Description: This function does not correspond to any function in the original `interrupted.rs`. It adds a stronger entry point requiring `spec_admission_time_valid()`. While useful for integration proofs, it expands the verified API surface beyond the original, which could confuse equivalence analysis.
   - Suggested Fix: Add a comment explicitly noting this is a verification-only helper not present in the original source, or move it to the proof file as a proof-mode wrapper.

### Low

1. **Per-thread state mutation (interrupt_reason) not modeled**
   - Location: `resume()` in `interrupted.rs` (exec); `spec_resume_reason_integration_obligation` in `interrupted.spec.rs`
   - Description: The original `InterruptedThread::resume()` calls `self.state.set_interrupt_reason(self.reason)` before converting to `ReadyThread`. Since threads are abstracted to integer IDs, this mutation is not modeled. The integration obligation `spec_resume_reason_integration_obligation` defines the contract, but it's not dischargeable within this module.
   - Suggested Fix: Acceptable for this module's abstraction level. Ensure the thread module's Verus verification discharges this obligation.

2. **ProcessState abstracted to PID only**
   - Location: `InterruptedProcess` struct in `interrupted.rs` (exec), line 134
   - Description: `Box<ProcessState>` is modeled as a single `Ghost<int>` PID. Other `ProcessState` fields (e.g., capabilities, memory mappings) are not tracked. The `state_mut()` accessor in the original allows mutation of these inner fields, but the verification model's frame condition holds trivially since only PID is tracked.
   - Suggested Fix: Document this as a known abstraction gap. If `ProcessState` becomes independently verifiable, add an invariant linking `self.pid@ == process_state.pid()`.

3. **`interrupt()` return type models extra information**
   - Location: Standalone `interrupt()` in `interrupted.rs` (exec), lines 551–557
   - Description: The original returns `InterruptedThread` (a single value). The verified version returns `(Ghost<int>, Ghost<int>)` — thread ID plus interrupt reason tag. The reason tag is additional information not surfaced by the original return type at this API level.
   - Suggested Fix: Minor modeling enrichment. Acceptable since it enables reasoning about interrupt reason propagation.

4. **`spec_no_duplicates` and `spec_seqs_disjoint` are duplicated between `InterruptedProcess` and `RunnableProcess`**
   - Location: `interrupted.spec.rs`, lines 206–216 (InterruptedProcess) and lines 363–373 (RunnableProcess)
   - Description: These identical helper specs are defined separately on both types. This is not a correctness issue but creates maintenance risk if one is updated without the other.
   - Suggested Fix: Consider factoring into a shared trait or free spec functions to avoid duplication.

## Positive Observations

- **Complete function coverage**: All 8 functions from the original source (including the standalone `interrupt()`) have verified counterparts. 8/8 coverage.
- **Verification passes cleanly**: 27/27 verification conditions pass with zero errors, no warnings.
- **No unjustified `assume` or `external_body`**: The code contains no `assume` statements and no `external_body` annotations. All proofs are completed without trusted escape hatches.
- **Comprehensive well-formedness invariant**: `wf()` captures non-empty interrupted threads, no-duplicate thread IDs within each list, and pairwise disjointness across all three lists — faithfully modeling Rust's ownership invariant for `NonEmptyVecDeque` containers.
- **Thread uniqueness proven**: `lemma_find_thread_result_unique` formally proves that under `wf()`, a thread ID can appear in at most one list. This is a key safety property.
- **Trust boundaries are thoroughly documented**: Every modeling gap (ghost-only model, per-thread state mutation, clock oracle, `find_thread` iterator search, `ProcessState` abstraction) is explicitly identified in the file header with formal integration obligations where applicable.
- **Clean spec/proof/exec separation**: The three-file split is well-organized — spec functions and View types in `.spec.rs`, all lemmas and proofs in `.proof.rs`, and struct definitions with exec functions in `.rs`.
- **Projection lemma for cross-module integration**: `lemma_project_to_runnable_boundary` provides a formal bridge to the sibling `runnable` module's boundary model, with all necessary invariants proven.
- **`resume()` postconditions are detailed and precise**: The postcondition captures PID preservation, the exact identity of the resumed thread, remaining thread list contents, sleeping/zombie preservation, and well-formedness of the result.
- **Integration obligations are machine-readable**: `spec_find_thread_integration_obligation` and `spec_resume_reason_integration_obligation` provide formal contracts that downstream verification can reference and discharge.

## Summary

This is a well-executed design-level verification of the `InterruptedProcess` module. All original functions are covered, the specifications are neither too weak (they capture non-trivial invariants like thread uniqueness and disjointness) nor too strong (they don't over-constrain the model). The proofs are complete with no trusted assumptions in code.

The primary limitation is inherent to the approach: this verifies a ghost model, not the executable Rust code. The gap between `Seq<int>` of thread IDs and real `NonEmptyVecDeque<InterruptedThread>` containers means container bugs (wrong element returned by `pop_front`, incorrect `from` conversion) are not caught. Similarly, `find_thread`/`find_thread_mut` are spec stubs that don't verify the actual iterator search.

These limitations are all clearly documented with explicit trust boundary markers and integration obligations. The verification provides strong confidence in the correctness of the state transition logic — threads are not lost or duplicated, PID is immutable, and well-formedness is preserved — which is the most valuable property for an OS kernel process state machine. The addition of `resume_with_valid_clock` as a verification-only helper is a minor API surface expansion that should be annotated as such.

**Recommendation**: Accept as-is for design-level verification. Track executable-code verification (with `external_body` wrappers for standard library containers) as future work to close the ghost-to-exec gap.
