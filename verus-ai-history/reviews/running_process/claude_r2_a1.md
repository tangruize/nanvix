# Review: running_process (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
None.

### High
None.

### Medium

- **Location:** `exit()` exec postcondition (exec:692–696, 717–721) and ghost construction (exec:726–733)
  **Description:** The zombie list ordering in the verified model does not match the original source. The original code at lines 189–200 constructs zombies as `[original_zombies..., running_zombie, ready_zombies...]` (push_back running onto existing, then append ready). The verified model constructs `seq![running_id].add(ready_ids).add(zombie_ids)` = `[running_zombie, ready_zombies..., original_zombies...]`. The postcondition asserts exact sequence equality (`==` on `Seq<int>`), which is order-sensitive. While count and set-membership properties are correct, the content-level spec is not semantically equivalent to the original code's behavior. Any downstream proof depending on element positions (e.g., `zombie_thread_ids@[0]`) would derive facts inconsistent with the real implementation.
  **Suggested Fix:** Reorder the ghost construction to match the original: `self.zombie_thread_ids@.push(self.running_thread_id@).add(self.ready_thread_ids@)`. Alternatively, weaken the postcondition to assert multiset equality (same elements, any order) rather than exact sequence equality.

### Low

- **Location:** `sleep()` Runnable branch outer postcondition (exec:582)
  **Description:** The outer ensures for `SleepResult::Runnable` asserts `rp.sleeping_thread_ids@.len() >= self.spec_sleeping_count() + 1`. Both sub-branches (ready at exec:588–589, interrupted at exec:594–595) establish exact equality via `==`. The outer `>=` is weaker than provable. This was flagged in round 1 and remains unfixed.
  **Suggested Fix:** Change `>=` to `==` on exec:582, or accept the weaker outer bound since inner implications provide exact equality.

- **Location:** `exit_thread()` inline comment (exec:876)
  **Description:** The comment reads "Known divergence: original passes self.zombie.take() (=None) here." in present tense, but the module header (exec:62–69) documents that the original source has already been fixed. The current original source at line 286 correctly passes `Some(zombie_threads)`. The present-tense comment is misleading about current state.
  **Suggested Fix:** Change to past tense: "Historical divergence: original *passed* self.zombie.take() (=None) here. Now fixed in source."

- **Location:** `wakeup()` precondition (exec:924)
  **Description:** The precondition `self.sleeping_count > 0 || !found` is logically redundant given `found == Self::spec_seq_contains(self.sleeping_thread_ids@, tid@)` (exec:922) and `wf()` (exec:921). If `found` is true, then `spec_seq_contains` is true, which requires `sleeping_thread_ids@.len() >= 1`, which by `wf()` means `sleeping_count >= 1`. So `sleeping_count > 0 || !found` is always satisfied. The precondition is harmless but adds unnecessary complexity.
  **Suggested Fix:** Remove the redundant precondition, or add a comment noting it's a solver hint.

## Positive Observations

- **Genuine bug discovery.** Verification identified and fixed a real bug in `exit_thread()` where `self.zombie.take()` after prior consumption always yielded `None`, losing the running thread's zombie state. This is precisely the kind of ownership/move-semantics bug that formal verification excels at catching.
- **Complete function coverage.** All 13 public/private functions from the original are modeled: 7 as verified exec (`new`, `get_tid`, `schedule`, `sleep`, `exit`, `exit_thread`, `wakeup`), 3 as spec-only models (`try_join_thread`, `find_thread`, `find_thread_mut`), and 3 as external_body accessors (`state`, `state_mut`, `running_mut`).
- **Content-level specifications.** State transitions specify exact thread list contents (not just counts) for all paths. The `schedule()`, `sleep()`, `exit_thread()`, and `wakeup()` postconditions all establish precise sequence equality for every thread list.
- **Well-constrained external_body.** The `interrupted_resume()` external_body has tight postconditions: PID preservation, wf(), sleeping/zombie passthrough, the specific ready thread being `interrupted[0]`, and thread count conservation. No unjustified `assume` anywhere.
- **Clean spec/proof/exec separation.** View types and spec functions are in the spec file, proof lemmas in the proof file, and executable logic in the exec file. The `include!` mechanism works cleanly.
- **Thorough trust boundary documentation.** Every elision (ContextInformation, Condvar, alarm, ExitStatus), oracle parameter (`found` in wakeup), and reference-type limitation (find_thread/find_thread_mut) is documented with rationale and caller obligations.
- **Well-formedness propagation.** Every function that produces a RunningProcess, RunnableProcess, SleepingProcess, or ZombieProcess ensures `wf()` in its postcondition.
- **Optional strict invariant.** The `wf_strict()` predicate providing thread ID uniqueness across all lists is a thoughtful addition that respects the trust boundary (Rust ownership) while enabling stronger downstream proofs.
- **Verification passes cleanly.** 40 verification conditions, 0 errors, in 6 seconds.

## Summary

The verification of `RunningProcess` is thorough and well-structured, covering all 13 original functions with appropriate modeling strategies for Verus's limitations. The single substantive issue is an ordering mismatch in `exit()`'s zombie list construction: the model builds `[running, ready, original_zombie]` while the original code produces `[original_zombie, running, ready]`. Since the postcondition uses exact sequence equality, this is a semantic equivalence gap. In practice, zombie lists are searched by ID (not indexed positionally), so the functional impact is low, but the content-level specification—the verification's strongest feature—is technically inaccurate for this function. The remaining issues are cosmetic (stale comment, weak bound, redundant precondition). The bug discovery in `exit_thread()` demonstrates real verification value. Overall, the verification provides strong assurance for the RunningProcess state machine, with the caveat that `exit()`'s zombie ordering spec should be corrected to match the original.
