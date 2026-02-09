# Review: interrupted_process (claude-opus-4.6)

## Grade: A-

## Previous Issue Disposition

### Medium #3 (resume_with_valid_clock not in original) — FIXED
The prover added `**Verification-only helper** — this function does NOT exist in the original source` to the doc comment (lines 428–432 of `interrupted.rs`). This was exactly the suggested fix. Verified via `git diff 52e98724..e26c8aaa`: only change is a 3-line doc comment update. Fix is genuine and complete.

### Medium #1 (Ghost-only verification) — ACKNOWLEDGED, NOT FIXED
No changes to address this. Remains an inherent limitation of the design-level verification approach. The module header documents this explicitly. Not fixable without `external_body` wrappers for standard library containers, which is a project-wide infrastructure effort.

### Medium #2 (find_thread/find_thread_mut spec-only stubs) — ACKNOWLEDGED, NOT FIXED
No changes. The `lemma_find_thread_refinement_assumption` and `spec_find_thread_integration_obligation` remain as trust boundary markers. This is a genuine Verus limitation (cannot express `Option<ThreadRef<'_>>` return types). Not fixable within current Verus capabilities.

### Low #1–#4 — ACKNOWLEDGED, NOT FIXED
No changes to any Low issues. All were already documented as acceptable design choices in the previous review.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

1. **Ghost-only (design-level) verification — no executable code verified** *(carried from r3_a1)*
   - Location: All functions in `interrupted.rs` (exec)
   - Description: All struct fields are `Ghost<...>`. The verification proves abstract state-transition logic but does NOT verify the actual Rust implementation. Container bugs (e.g., `NonEmptyVecDeque::pop_front` returning wrong element, `NonEmptyVecDeque::from` mishandling empty input) are not caught.
   - Suggested Fix: Future work — requires `external_body` wrappers for `NonEmptyVecDeque`/`VecDeque` and exec-mode functions.

2. **`find_thread`/`find_thread_mut` are spec-only stubs** *(carried from r3_a1)*
   - Location: `find_thread()` / `find_thread_mut()` in `interrupted.rs` (exec), lines 500–531
   - Description: Both return `Ghost(self.spec_find_thread(tid@))` directly. The original's `iter().find(|t| t.id() == tid)` search across three collections (interrupted → sleeping → zombie) is unverified. Integration obligation and refinement assumption are documented but not dischargeable in this module.
   - Suggested Fix: Verus limitation. Replace with verified implementation if Verus later supports reference-typed returns.

### Low

1. **Per-thread state mutation (interrupt_reason) not modeled** *(carried from r3_a1)*
   - Location: `resume()` in `interrupted.rs`; `spec_resume_reason_integration_obligation` in `interrupted.spec.rs`
   - Description: `InterruptedThread::resume()` sets `interrupt_reason` before converting to `ReadyThread`. Not modeled since threads are integer IDs. Integration obligation defined.
   - Suggested Fix: Thread module's Verus verification should discharge this obligation.

2. **ProcessState abstracted to PID only** *(carried from r3_a1)*
   - Location: `InterruptedProcess` struct, line 134
   - Description: `Box<ProcessState>` → `Ghost<int>` PID. Other ProcessState fields untracked. `state_mut()` frame condition is trivially satisfied.
   - Suggested Fix: Add linking invariant if ProcessState gets independent verification.

3. **`interrupt()` return type models extra information** *(carried from r3_a1)*
   - Location: Standalone `interrupt()`, lines 553–558
   - Description: Returns `(Ghost<int>, Ghost<int>)` (ID + reason tag) while original returns `InterruptedThread`. Minor modeling enrichment enabling interrupt reason reasoning.
   - Suggested Fix: Acceptable as-is.

4. **`spec_no_duplicates` and `spec_seqs_disjoint` duplicated across types** *(carried from r3_a1)*
   - Location: `interrupted.spec.rs`, lines 206–216 vs. 363–373
   - Description: Identical helper specs on `InterruptedProcess` and `RunnableProcess`. Maintenance risk if one is updated without the other.
   - Suggested Fix: Factor into free spec functions if Verus module structure allows.

## New Issues Introduced

- None. The only change was a 3-line doc comment update that introduces no new verification or semantic issues.

## Verification Results

- **27/27** verification conditions pass, 0 errors.
- **No `assume`, `external_body`, or `trusted` annotations** in any of the three files.
- Verification completes in ~6 seconds.

## Positive Observations

- **Complete function coverage**: All 8 original functions have verified counterparts (new, from_sleeping, state, state_mut, resume, find_thread, find_thread_mut, interrupt). 8/8 coverage.
- **`resume()` correctness is thoroughly proven**: Front thread is popped and becomes the sole ready thread (ID-preserving). Remaining interrupted threads form the tail. Sleeping and zombie threads are preserved. PID is immutable. RunnableProcess wf() is established. The proof handles all 6 pairwise disjointness conditions and 4 no-duplicates conditions for the output.
- **Thread uniqueness under wf()**: `lemma_find_thread_result_unique` proves each thread ID appears in at most one list. This is a critical safety property for a kernel process state machine.
- **Clean trust boundary documentation**: Every abstraction gap is explicitly documented with formal integration obligations where applicable. The prover did not hide any assumptions.
- **Projection lemma for cross-module linking**: `lemma_project_to_runnable_boundary` bridges to the sibling `runnable` module's boundary model with invariants proven.
- **No soundness holes**: No escape hatches used anywhere in the verification.

## Summary

The prover addressed the one actionable issue from the previous review (adding verification-only annotation to `resume_with_valid_clock`). The remaining 6 issues are all inherent limitations of the design-level verification approach or Verus tooling constraints — they cannot be fixed within the scope of this module and were already documented.

The verification quality is solid: 27/27 conditions pass, no trusted assumptions, complete function coverage, and the key safety properties (thread uniqueness, PID immutability, well-formedness preservation, correct resume semantics) are all proven. The state-transition logic for `InterruptedProcess` is verified at the design level with appropriate trust boundary markers for what lies outside scope.

**Grade rationale**: A- (unchanged). The verification is well-executed within its stated scope. The ghost-only limitation prevents an A grade — the gap between the abstract `Seq<int>` model and real `NonEmptyVecDeque` containers is a meaningful verification gap for a kernel component. However, all trust boundaries are explicitly documented, and the abstract correctness proofs provide genuine confidence in the state machine logic.
