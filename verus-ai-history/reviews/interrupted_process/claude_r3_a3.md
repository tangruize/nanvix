# Review: interrupted_process (claude-opus-4.6)

## Grade: A-

## Previous Issue Disposition

**No changes were made.** All three files (interrupted.rs, interrupted.spec.rs, interrupted.proof.rs) are byte-identical to commit e26c8aaa reviewed in round r3_a2. The prover did not address any of the 6 remaining issues from the previous review.

### Medium #1 (Ghost-only verification) — UNCHANGED
No changes. Remains an inherent design-level limitation.

### Medium #2 (find_thread/find_thread_mut spec-only stubs) — UNCHANGED
No changes. Remains a Verus tooling limitation.

### Low #1–#4 — UNCHANGED
No changes to any Low issues.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

1. **Ghost-only (design-level) verification — no executable code verified** *(carried from r3_a1, unchanged)*
   - Location: All functions in `interrupted.rs` (exec)
   - Description: All struct fields are `Ghost<...>`. The verification proves abstract state-transition logic but does NOT verify the actual Rust implementation. Container bugs (e.g., `NonEmptyVecDeque::pop_front` returning wrong element, `NonEmptyVecDeque::from` mishandling empty input) are not caught.
   - Suggested Fix: Future work — requires `external_body` wrappers for standard library containers.

2. **`find_thread`/`find_thread_mut` are spec-only stubs** *(carried from r3_a1, unchanged)*
   - Location: `find_thread()` / `find_thread_mut()` in `interrupted.rs` (exec), lines 500–531
   - Description: Both return `Ghost(self.spec_find_thread(tid@))` directly. The original's `iter().find(...)` search logic is unverified. Integration obligation documented but not dischargeable within this module.
   - Suggested Fix: Verus limitation. Replace with verified implementation if Verus later supports reference-typed returns.

### Low

1. **Per-thread state mutation (interrupt_reason) not modeled** *(carried, unchanged)*
   - Location: `resume()` in exec; `spec_resume_reason_integration_obligation` in spec
   - Description: Thread module must discharge this obligation.

2. **ProcessState abstracted to PID only** *(carried, unchanged)*
   - Location: `InterruptedProcess` struct, line 134
   - Description: `Box<ProcessState>` → `Ghost<int>` PID. Other fields untracked.

3. **`interrupt()` return type models extra information** *(carried, unchanged)*
   - Location: Standalone `interrupt()`, lines 553–558
   - Description: Returns `(Ghost<int>, Ghost<int>)` vs original's `InterruptedThread`.

4. **`spec_no_duplicates`/`spec_seqs_disjoint` duplicated across types** *(carried, unchanged)*
   - Location: `interrupted.spec.rs`, lines 206–216 vs. 363–373
   - Description: Identical helper specs on both `InterruptedProcess` and `RunnableProcess`.

## New Issues Introduced

- None. No changes were made.

## Verification Results

- **27/27** verification conditions pass, 0 errors.
- **No `assume`, `external_body`, or `trusted`** annotations in any file.
- Verification completes in ~6 seconds.

## Positive Observations

- All 8 original functions covered (new, from_sleeping, state, state_mut, resume, find_thread, find_thread_mut, interrupt).
- `resume()` has thorough postconditions: front thread popped, becomes sole ready thread, tail preserved, sleeping/zombie preserved, PID immutable, RunnableProcess wf() established.
- Thread uniqueness proven via `lemma_find_thread_result_unique`.
- All trust boundaries explicitly documented with formal integration obligations.
- No soundness holes — no escape hatches used anywhere.

## Summary

No changes were made since the previous review (r3_a2). All 6 remaining issues are carried forward unchanged. The grade remains **A-**.

The 2 Medium issues (ghost-only verification, spec-only find_thread stubs) are inherent limitations of the design-level verification approach and Verus tooling — they cannot be fixed within this module's scope. The 4 Low issues are acceptable design choices that are properly documented. All issues have been stable across three review rounds, and the prover has demonstrated they understand the trust boundaries.

**Grade rationale**: A- (unchanged across all three rounds). The verification is well-executed within its stated scope. The ghost-only limitation prevents a higher grade since the gap between abstract `Seq<int>` and real `NonEmptyVecDeque` containers is meaningful for a kernel component. However, the verification provides genuine confidence in the state-transition logic with all trust boundaries explicitly documented.

**Recommendation**: Accept. No further rounds needed — the remaining issues require project-wide infrastructure (external_body wrappers) or Verus language improvements, not changes to this module.
