# Review: running_process (claude-opus-4.6)

## Grade: A+

## Previous Issue Disposition

### Medium #1 (exit() zombie list ordering mismatch): ✅ FIXED
The ghost construction at exec:728–729 now uses `self.zombie_thread_ids@.push(self.running_thread_id@).add(self.ready_thread_ids@)`, producing `[original_zombies..., running_zombie, ready_zombies...]`. This matches the original source exactly: line 189–191 takes existing zombies and `push_back`s the running zombie, then line 199–200 appends the terminated ready threads. The postconditions at exec:694–696 and exec:719–721 both use this corrected ordering. The proof lemma `lemma_exit_zombie_content` (proof:193–205) was also updated to match. Verified by tracing the original code step-by-step.

### Low #1 (sleep() Runnable branch outer sleeping bound uses >=): ✅ FIXED
Exec line 582 now asserts `rp.sleeping_thread_ids@.len() == self.spec_sleeping_count() + 1` (exact equality), matching the inner branch specifications. This makes the outer bound consistent with the strong content-level specs used throughout.

### Low #2 (exit_thread() inline comment stale present tense): ✅ FIXED
Exec line 872 now reads "Historical: original passed self.zombie.take() (=None) here. Now fixed in source." — correctly uses past tense to reflect that the original source has been patched.

### Low #3 (wakeup() redundant precondition): ✅ ADDRESSED
The precondition at exec:922 remains (`self.sleeping_count > 0 || !found`) but is now annotated with a comment at exec:920–921 explaining it is a solver hint for Verus arithmetic. This is acceptable — redundant preconditions that aid the solver are a common Verus pattern, and the comment prevents future confusion.

## Issues Found

### Critical
None.

### High
None.

### Medium
None.

### Low
None.

## Positive Observations

- **All 4 issues from round 2 are resolved.** The zombie ordering fix was the only substantive change, and it was done correctly — both the ghost construction, the postconditions, and the proof lemma were updated in sync.
- **Complete function coverage.** All 13 original functions are modeled: 7 verified exec (`new`, `get_tid`, `schedule`, `sleep`, `exit`, `exit_thread`, `wakeup`), 3 spec-only (`try_join_thread`, `find_thread`, `find_thread_mut`), 3 external_body accessors (`state`, `state_mut`, `running_mut`).
- **Content-level specifications throughout.** Every state transition specifies exact thread list contents via sequence equality, not just counts. The zombie ordering now faithfully matches the original source's `push_back` + `append` semantics.
- **Sound external_body usage.** The single `interrupted_resume()` external_body has tight postconditions: PID preservation, wf(), sleeping/zombie passthrough, specific ready thread identity, and thread count conservation. No unjustified `assume` anywhere in the module.
- **Genuine bug discovery.** The `exit_thread()` zombie loss bug remains a high-value finding where formal verification caught a real ownership/move-semantics error.
- **Clean spec/proof/exec separation.** View types and spec functions in `.spec.rs`, proof lemmas in `.proof.rs`, executable logic in `.rs`. All three files are internally consistent.
- **Thorough trust boundary documentation.** Every elision, oracle parameter, and modeling limitation is documented with rationale and caller obligations.
- **Verification passes cleanly.** 40 verification conditions, 0 errors, in 5 seconds.

## Summary

The verification of `RunningProcess` is now comprehensive, sound, and semantically faithful to the original source. The zombie ordering fix was the last substantive gap — the model now correctly specifies `[original_zombies, running_zombie, ready_zombies]` ordering in `exit()`, matching the original's `push_back` + `append` sequence. All previous issues across both review rounds are fully resolved. The verification provides strong assurance for the RunningProcess state machine: all state transitions are specified at the content level (exact sequence equality), PID immutability is maintained across all operations, well-formedness propagates through all transitions, and the single external_body dependency is tightly constrained. No remaining issues.
