# Review: running_process (claude-opus-4.6)

## Grade: A-

## Previous Issue Disposition

### Critical #1 (InterruptedProcess drops sleeping threads in sleep()): ✅ FIXED
The `InterruptedProcess` struct now includes `sleeping_thread_ids` (exec:158–159). In `sleep()`'s interrupted branch (exec:447–451), `new_sleeping_ids` is correctly passed. The `interrupted_resume()` external_body now ensures `result.sleeping_thread_ids@ == ip.sleeping_thread_ids@` (exec:233). The `sleep()` spec now asserts exact sleeping thread content for the interrupted branch (exec:402–405). Verified: sleeping threads are no longer lost.

### Critical #2 (InterruptedProcess lacks sleeping field for exit()): ✅ FIXED
The `InterruptedProcess` struct now has `sleeping_thread_ids`. In `exit()` (exec:552), `Ghost(Seq::empty())` is correctly passed — matching the original where `self.sleeping_threads` was already taken. The spec correctly asserts `rp.sleeping_thread_ids@.len() == 0` (exec:502).

### High #1 (exit_thread() divergence undocumented): ✅ FIXED
Thoroughly documented in three places: module header (exec:62–70), function doc comment (exec:579–585), and inline comment (exec:667–668). I verified the claim: at original source line 261, `self.zombie.take()` moves the zombie list into `zombie_threads`. At line 286, `self.zombie.take()` returns `None` because it was already consumed. The `zombie_threads` local (containing the exited thread) is not passed and is dropped. The divergence claim is correct and well-documented.

### High #2 (Missing state()/state_mut()/running_mut()): ✅ FIXED
Now documented as trust boundaries: `running_mut()` at exec:56–58 and spec:65–69 with obligation that callers must preserve running thread ID. `state()/state_mut()` at exec:54–55 and spec:70–72 with PID immutability obligation.

### High #3 (Missing try_join_thread()): ✅ FIXED
`spec_try_join_thread()` added (spec:257–286) with correct classification: tag 1 for running (OperationNotPermitted), tag 0 for zombie (removal), tag 2 for live threads (condvar), tag 3 for not found (NoSuchProcess). Search order matches original. Three proof lemmas added (proof:345–371). Documented in exec header (line 26, 51–53).

### Medium #1 (sleep() Runnable spec weak for interrupted branch): ✅ FIXED
Spec now asserts exact sleeping thread content for both ready and interrupted branches (exec:391–405), plus zombie preservation (exec:393).

### Medium #2 (exit() Runnable spec weak for zombie content): ✅ FIXED
Spec now asserts exact zombie content: `rp.zombie_thread_ids@ == seq![self.running_thread_id@].add(self.ready_thread_ids@).add(self.zombie_thread_ids@)` (exec:498–500), plus `rp.sleeping_thread_ids@.len() == 0` (exec:502).

### Medium #3 (wakeup() oracle undocumented): ✅ FIXED
Documented in spec file header (spec:54–58) and function doc comment (exec:700–706).

### Medium #4 (find_thread()/find_thread_mut() undocumented): ✅ FIXED
Documented as trust boundary in spec header (spec:59–64) and exec header (exec:49–50).

### Medium #5 (alarm parameter dropped undocumented): ✅ FIXED
Documented in spec header (spec:46–49), exec header (exec:39), and function doc (exec:374–375).

### Low #1 (Trivially true proof lemmas): ✅ FIXED
Lemmas are now substantive: `lemma_schedule_result_has_ready` proves `new_ready.len() >= 1` (proof:91–95); `lemma_schedule_preserves_total_threads` proves actual total count equality (proof:109–115); sleep lemmas prove content facts; new lemmas `lemma_exit_zombie_content` (proof:172–183), `lemma_exit_thread_zombie_content` (proof:217–227), and `lemma_wakeup_preserves_total_count` (proof:269–285) are substantive. One lemma (`lemma_exit_no_interrupted_gives_zombie`, proof:202–210) is still marginal — it ensures `!(interrupted > 0 || sleeping > 0)` which is just the negation of the requires — but is acceptable.

### Low #2 (wf() lacks uniqueness): ✅ FIXED
`wf_strict()` added (spec:321–341) with pairwise disjointness of all lists plus running thread exclusion. Uses `spec_seqs_disjoint` helper (spec:300–305).

### Low #3 (EXIT_STATUS_INTERRUPTED unused): ✅ FIXED
Constant removed from spec file.

### Low #4 (pub fields undocumented): ✅ FIXED
Documented in exec header (exec:72–76).

## Issues Found

### Critical
None.

### High
None.

### Medium

- **Location:** `spec_try_join_thread()` in spec file (spec:270–286)
  **Description:** The spec models `try_join_thread()` as a pure classification function returning an int tag, but the original function has a side effect: it removes the zombie thread from the zombie list via `self.zombie.take()` + `remove_if()` (original source lines 350–359, the struct is mutated via `&mut self`). The spec captures *what happens* (tag 0 = zombie found) but not the *resulting state* (zombie list shrinks by 1). Downstream callers that depend on the zombie list being modified after a join will find no spec support for this.
  **Suggested Fix:** Add a companion spec function (e.g., `spec_try_join_thread_post`) that models the resulting zombie list after a successful join: `self.zombie_thread_ids@ == old.zombie_thread_ids@ with tid removed`. Or extend the return type to include the resulting state.

- **Location:** `sleep()` spec (exec:401–405), interrupted branch — ready/interrupted thread content unspecified
  **Description:** In the interrupted branch of `sleep()`, the sleeping and zombie lists are specified exactly, but `rp.ready_thread_ids@` and `rp.interrupted_thread_ids@` are not constrained beyond `wf()` (ready >= 1). This is because `interrupted_resume()` is external_body and only guarantees PID, wf, sleeping, and zombie preservation — it doesn't specify how interrupted threads map to ready threads. While this is an inherent limitation of the boundary model, it means a caller cannot reason about which specific thread became the running thread after resume.
  **Suggested Fix:** Strengthen `interrupted_resume()` postcondition to relate `result.interrupted_thread_ids@` and `result.ready_thread_ids@` to `ip.interrupted_thread_ids@`. For example: `result.ready_thread_ids@.len() + result.interrupted_thread_ids@.len() == ip.interrupted_thread_ids@.len()`. This would enable total-thread-count preservation proofs through the interrupted path.

### Low

- **Location:** `lemma_exit_no_interrupted_gives_zombie` in proof file (proof:202–210)
  **Description:** This lemma's ensures clause (`!(self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0)`) is directly implied by its requires (`self.spec_interrupted_count() == 0 && self.spec_sleeping_count() == 0`). It proves nothing beyond what Verus already knows from the precondition. All other lemmas have been meaningfully strengthened.
  **Suggested Fix:** Strengthen to prove something useful, e.g., that `self.interrupted_thread_ids@.add(self.sleeping_thread_ids@).len() == 0`, or remove.

## Positive Observations

- **Comprehensive issue resolution.** All 14 issues from the previous review were addressed — 13 fully fixed, 1 marginally improved. No issues were dismissed without justification.
- **InterruptedProcess boundary model is now complete.** The addition of `sleeping_thread_ids` with proper threading through `interrupted_resume()` closes the most critical gap. The `exit()` path correctly passes `Seq::empty()` while `sleep()` passes the actual sleeping list — matching the original's behavior precisely.
- **Content-level specifications are now strong.** `schedule()`, `sleep()`, `exit()`, `exit_thread()`, and `wakeup()` all specify exact thread list contents (not just counts) for their primary paths. This is significantly stronger than the previous version.
- **Excellent documentation of the exit_thread() divergence.** The claim that `self.zombie.take()` at original line 286 returns `None` (because `self.zombie` was consumed at line 261) is verified correct. This is a genuine bug finding in the original source. The three-location documentation (header, doc comment, inline) is thorough.
- **wf_strict() is well-designed.** Pairwise disjointness across all 6 list pairs plus running thread exclusion provides a complete uniqueness predicate for downstream proofs.
- **spec_try_join_thread() classification is correct.** The search order matches the original (running → zombie → ready/sleeping/interrupted → not found), and the four tags correctly map to the original's complex nested Result return type.
- **Proof lemmas are now substantive.** Most lemmas prove non-trivial properties about sequence lengths, content, and total count preservation. The new `lemma_wakeup_preserves_total_count` and `lemma_exit_zombie_content` are particularly valuable.
- **Trust boundaries are clearly delineated.** The spec file header (lines 44–72) comprehensively documents all trust assumptions: oracle parameter, reference-returning functions, mutable accessor obligations, alarm elision, and Condvar elision.
- **Verification passes cleanly.** 37 verification conditions, 0 errors — up from 32, reflecting the additional proof content.

## Summary

The prover addressed all 14 issues from the previous review thoroughly and correctly. The two critical issues (sleeping thread data loss through `InterruptedProcess`) are fully resolved with structural fixes to the boundary model and `interrupted_resume()` postconditions. The specifications are now significantly stronger with content-level assertions throughout. Trust boundaries are comprehensively documented. The remaining issues are minor: `spec_try_join_thread()` doesn't model the zombie removal side effect, the `interrupted_resume()` external_body could constrain ready/interrupted thread content more tightly, and one proof lemma is still trivially true. The verification provides solid assurance for the RunningProcess state machine, correctly identifying a genuine bug in the original source (`exit_thread()` zombie loss) while faithfully modeling all other behavior.
