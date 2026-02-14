# Review: running_process Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High

- **Public method specs reference concrete fields directly (Step 3 violation).**
  The methodology (Step 3) states: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`."
  Multiple public method ensures/requires clauses reference `self.ready_thread_ids@`, `self.zombie_thread_ids@`, `self.interrupted_thread_ids@`, `self.sleeping_thread_ids@`, `self.ready_count`, `self.zombie_count`, `self.running_thread_id`, `self.pid`, etc.
  These should use `self@.ready_thread_ids`, `self@.zombie_thread_ids`, etc. (view-level fields).
  Affected methods: `try_join_thread` (lines 564–581), `state_mut` (lines 488–492), `running_mut` (lines 515–518), `find_thread_mut` (lines 720–724), `wakeup` (lines 1192–1200), `schedule` (lines 748–752), `sleep` (lines 815–828), `exit` (lines 933–945), `exit_thread` (lines 1048–1088).
  This is a pervasive pattern — virtually all public method specs expose concrete implementation fields rather than view abstractions.

- **Spec helper functions on `RunningProcess` are `pub open` instead of private or on the View type (Step 3 violation).**
  Functions like `spec_pid`, `spec_running_thread_id`, `spec_ready_count`, `spec_sleeping_count`, `spec_interrupted_count`, `spec_zombie_count`, `spec_has_ready_thread`, `spec_has_sleeping_thread`, `spec_has_zombie_thread`, `spec_has_interrupted_thread`, `spec_has_thread`, `spec_find_thread`, `spec_try_join_thread`, `spec_try_join_zombie_post`, `spec_seq_contains`, `spec_remove_at`, `spec_seqs_disjoint`, `wf_strict`, and `mutation_frame_preserved` are all `pub open spec fn` on `RunningProcess` (lines 146–309).
  Per Step 3: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." These should either be private spec functions on `RunningProcess`, or (for abstraction-level properties) `pub open spec fn` on `RunningProcessView`.

### Medium

- **View types use `Seq<int>` but some exec-level specs use `Seq<u64>` (partial mismatch).**
  The View types correctly use abstract types (`int` for identifiers, `Seq<int>` for thread ID lists) — this follows the methodology.
  However, many spec functions on `RunningProcess` (e.g., `spec_seq_contains`, `spec_remove_at`, `spec_seqs_disjoint`, `spec_try_join_zombie_post`) operate on `Seq<u64>` (lines 256–278), mixing concrete types into specification logic.
  Per Step 1, spec functions should use abstract types. These helpers should operate on `Seq<int>` if they are meant to be part of the public spec surface.

- **`mutation_frame_preserved` (line 298) references concrete fields.**
  This spec function on `RunningProcess` references `new_self.ready_count`, `new_self.interrupted_count`, `new_self.sleeping_count`, `new_self.zombie_count`, and `new_self.ready_thread_ids@`, etc. — all concrete implementation fields. This should either be a private spec function or reformulated on the View type.

### Low

- **`view()` is `closed spec fn` without `pub` keyword.**
  The View trait implementation (lines 593–604) declares `closed spec fn view()` without `pub`. The methodology says `pub closed spec fn view()`. In Verus, trait method visibility is typically inherited from the trait, so this may be fine in practice, but it deviates from the stated convention. Same applies to all other process types' View impls (lines 608, 621, 632, 644).

- **No `wf()` function — uses `inv()` and `wf_strict()` instead.**
  The methodology references `inv()/wf()`. This module uses `inv()` as `pub closed spec fn` (correct) and `wf_strict()` as `pub open spec fn` (should be private or on the View type per Step 3). The `wf_strict` includes disjointness properties that are abstraction-level concerns and could be on `RunningProcessView`.

### Info

- **`external_body` usages are justified and documented.**
  Four `external_body` functions exist: `interrupted_resume` (line 347), `state` (line 462), `state_mut` (line 481), `running_mut` (line 508). All are clearly documented with trust boundaries, frame conditions, and discharge criteria. `interrupted_resume` is a cross-module boundary function. `state`/`state_mut`/`running_mut` return references that Verus cannot model directly. These are reasonable trust assumptions.

- **No `assume` or `admit` found.** Clean of unjustified assumptions.

- **Verification passes: 44 verified, 0 errors.**

- **View types use abstract types correctly.**
  `RunningProcessView`, `RunnableProcessView`, `SleepingProcessView`, `InterruptedProcessView`, and `ZombieProcessView` all use `int` for identifiers and `Seq<int>` for thread ID sequences — this is correct per Step 1.

- **Abstract state transition functions on `RunningProcessView` are well-structured.**
  The spec file (lines 385–582) includes comprehensive spec transition functions (`spec_schedule`, `spec_sleep_*`, `spec_exit_*`, `spec_wakeup_*`, `spec_join_*`) with connecting lemmas in the proof file. This is good practice for downstream reasoning.

- **`inv()` is correctly `pub closed spec fn` for all process types.**
  `RunningProcess::inv()` (line 138), `RunnableProcess::inv()` (line 323), `SleepingProcess::inv()` (line 339), `InterruptedProcess::inv()` (line 355), `ZombieProcess::inv()` (line 375) — all correct.

## Summary

The verification is complete and sound (44 verified, 0 errors, no assume/admit). View types are well-designed with abstract types (`int`, `Seq<int>`), `view()` is correctly `closed`, and `inv()` is `pub closed spec fn` on all process types. The abstract state transition functions and connecting lemmas provide strong spec-level reasoning support.

The primary methodology gap is **pervasive leakage of concrete implementation fields into public method specifications** (Step 3 violation). Public ensures/requires clauses consistently reference `self.ready_thread_ids@`, `self.zombie_count`, etc. instead of `self@.ready_thread_ids`, `self@.zombie_thread_ids`, etc. Additionally, numerous spec helper functions are declared as `pub open spec fn` on `RunningProcess` rather than being private (for implementation use) or placed on `RunningProcessView` (for abstraction use), violating the Step 3 guideline that only `inv` and `view` should be public spec functions on the exec type.

These are information-hiding issues that don't affect verification soundness but would cause downstream consumers to depend on implementation details rather than the abstraction, defeating the purpose of the View pattern. Fixing these would bring the module to A-grade methodology compliance.
