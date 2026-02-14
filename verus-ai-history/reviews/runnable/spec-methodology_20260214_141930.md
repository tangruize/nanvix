# Review: runnable Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **View types use `Seq<i64>` instead of `Seq<int>` for thread IDs.** The methodology (Step 1) states View types should use abstract types (`int`, `Seq<int>`) rather than concrete types. `RunnableProcessView` uses `Seq<i64>` for `ready_thread_ids`, `ready_admission_times`, `interrupted_thread_ids`, `sleeping_thread_ids`, `zombie_thread_ids`. Similarly `RunningProcessView`, `InterruptedProcessView`, `ZombieProcessView` all use `Seq<i64>`. These should be `Seq<int>` to fully abstract away the concrete `i64` representation. The `pid` field correctly uses `int`, but the thread ID and admission time sequences do not follow the same principle.
  - Files: `runnable.spec.rs:118-131`, `runnable.spec.rs:139-155`, `runnable.spec.rs:158-166`, `runnable.spec.rs:168-177`

- **Public method specs use `self.field` instead of `self@.field`.** The methodology (Step 3) states public method specs should never directly reference `self` fields — use `self@.field` (i.e., `self.view().field`) instead. Multiple public methods reference concrete fields directly in their `ensures` clauses:
  - `run()` (line 423-431): `self.ready_thread_ids@[sel]`, `self.interrupted_thread_ids@`, `self.sleeping_thread_ids@`, `self.zombie_thread_ids@`
  - `terminate()` (line 506-531): `self.interrupted_thread_ids@`, `self.sleeping_thread_ids@`, `self.ready_thread_ids@`, `self.zombie_thread_ids@`
  - `wakeup()` (line 603-629): `self.ready_thread_ids@`, `self.ready_admission_times@`, `self.sleeping_thread_ids@`, `self.interrupted_thread_ids@`, `self.zombie_thread_ids@`
  - `add_thread()` (line 744-749): `self.ready_thread_ids@`, `self.ready_admission_times@`, `self.interrupted_thread_ids@`, `self.sleeping_thread_ids@`, `self.zombie_thread_ids@`
  - Similarly `result.ready_thread_ids@`, `result.interrupted_thread_ids@`, etc. access fields on result types directly rather than through views.
  - File: `runnable.rs`

### Medium
- **`view()` is `open spec fn` instead of `pub closed spec fn`.** The methodology (Step 1) says `view()` should be `pub closed spec fn`. The code (line 700-704 of `runnable.spec.rs`) explains this is because the Verus `View` trait requires `open`. This is a justified deviation documented with a clear rationale. However, this means the View type fields serve as the abstraction boundary rather than the `view()` function itself, making the `Seq<i64>` vs `Seq<int>` issue more impactful since the concrete `i64` type leaks through the open view.

- **Many spec functions on `RunnableProcess` are `pub open` beyond `view()` and `wf()`.** The methodology (Step 3) says "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." Functions like `spec_pid`, `spec_ready_count`, `spec_interrupted_count`, `spec_sleeping_count`, `spec_zombie_count`, `spec_total_thread_count`, `spec_ready_thread_id`, `spec_ready_admission_time`, `spec_has_ready_thread`, `spec_has_sleeping_thread`, `spec_has_thread`, `spec_has_interrupted_thread`, `spec_has_zombie_thread`, `spec_find_thread`, `spec_seq_contains`, `spec_remove_at`, `spec_min_index_rec`, `spec_earliest_ready_index`, `spec_earliest_admission_time`, `spec_seqs_disjoint`, `spec_ids_disjoint` are all `pub open spec fn` on the exec type. Most of these should be on `RunnableProcessView` instead, or made private. Some like `spec_pid()`, `spec_ready_count()` provide convenient abstractions and could be justified on the View type; the helpers like `spec_remove_at`, `spec_seq_contains`, `spec_min_index_rec` are utility functions that don't need to be public on the exec type.
  - File: `runnable.spec.rs:213-426`

### Low
- **`clock_now()` is `external_body` — justified and documented.** The single `external_body` in the exec file (line 89) models `clock::now()` as returning a non-negative `i64`. This is a HAL boundary function with a minimal postcondition. Well-documented in the trust boundary section. No `assume` or `admit` patterns found. This is acceptable.

- **`RunnableProcessView::wf()` is `pub open spec fn` (line 519).** The methodology says `inv()/wf()` should be `pub closed spec fn`. The exec-level `RunnableProcess::wf()` is correctly `pub closed spec fn` (line 387). The View-level `wf()` being open is reasonable since it operates on abstract state and serves a different purpose (enabling downstream reasoning on the abstract model), but worth noting as a minor deviation.

## Summary

The runnable verification module is well-structured with 67 verified items, 0 errors, and no assume/admit patterns. The single `external_body` (`clock_now`) is justified and documented. The proof file contains comprehensive lemmas including bridging lemmas tying exec results to View-level abstract state transitions, which is excellent practice.

The main methodology gaps are: (1) View types use `Seq<i64>` rather than fully abstract `Seq<int>`, leaking the concrete `i64` representation into the abstract model; (2) public method specifications reference `self.field@` (concrete field access) instead of `self@.field` (view-based access); and (3) there are many `pub open spec fn` functions on the exec type that should either live on the View type or be private. These are systematic issues that would require a coordinated refactoring to address — changing `Seq<i64>` to `Seq<int>` in View types would cascade through all spec functions and proof lemmas.

The `wf()` on `RunnableProcess` is correctly `pub closed spec fn`, the `view()` deviation to `open` is justified by trait requirements, and the overall verification coverage is thorough with content-level postconditions, PID immutability proofs, and complete bridging lemmas for all state transitions.
