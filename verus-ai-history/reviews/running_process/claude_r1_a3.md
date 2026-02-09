# Review: running_process (claude-opus-4.6)

## Grade: A

## Previous Issue Disposition

### Medium #1 (spec_try_join_thread() zombie removal side effect unmodeled): ✅ FIXED
`spec_try_join_zombie_post()` added (spec:298–307) modeling the resulting zombie sequence after removal. Uses `choose` to pick a matching index and `spec_remove_at` to produce the post-state sequence. `lemma_try_join_zombie_post_shrinks` (proof:381–395) proves the zombie list shrinks by exactly 1, using `lemma_remove_at_length` for the proof. The spec correctly handles the nondeterminism of which duplicate to remove (if any). The side-effect model is sound.

### Medium #2 (interrupted_resume() postcondition too weak for ready/interrupted content): ✅ FIXED
The `interrupted_resume()` external_body (exec:228–247) now includes two new postconditions:
- `result.ready_thread_ids@.len() + result.interrupted_thread_ids@.len() == ip.interrupted_thread_ids@.len()` (exec:241–242) — thread count conservation through resume.
- `result.ready_thread_ids@.len() >= 1` (exec:244) — at least one interrupted thread becomes ready.

This is exactly the constraint I suggested. It enables downstream callers to reason about total thread count preservation through the interrupted resume path. Verified that Verus accepts these postconditions (38 verification conditions pass).

### Low #1 (lemma_exit_no_interrupted_gives_zombie trivially true): ✅ FIXED (marginal)
The ensures now proves `combined_interrupted.len() == 0` where `combined_interrupted = self.interrupted_thread_ids@.add(self.sleeping_thread_ids@)`. While the individual `len() == 0` facts still follow from the requires, the additional proof that the concatenation of the two empty sequences is also empty (a cross-cutting assertion) provides marginal value for downstream consumers who need the combined form. Acceptable.

## Issues Found

### Critical
None.

### High
None.

### Medium
None.

### Low

- **Location:** `sleep()` spec (exec:400), Runnable branch — outer sleeping bound uses `>=`
  **Description:** The outer ensures for `SleepResult::Runnable` asserts `rp.sleeping_thread_ids@.len() >= self.spec_sleeping_count() + 1`. Both sub-branches (ready and interrupted) prove exact equality via inner implications (exec:406–407 and exec:412–413 respectively, both using `==`). The outer `>=` is weaker than provable. This is a cosmetic issue — any caller using the branch conditions can derive exact equality — but the outer bound could be strengthened to `==` for consistency with the strong content-level specs elsewhere.
  **Suggested Fix:** Change `>=` to `==` on exec:400, or leave as-is since the inner implications already establish exact equality.

## Positive Observations

- **All 17 issues from rounds 1–2 are now resolved.** The prover systematically addressed every issue across three rounds, with no dismissals and no regressions.
- **`interrupted_resume()` postcondition is now well-constrained.** The thread count conservation property (`ready + interrupted = ip.interrupted`) is the right abstraction — it enables total-count reasoning without over-committing to implementation details of the sibling module.
- **`spec_try_join_zombie_post()` correctly models the mutation.** The combination of a classification function (`spec_try_join_thread`) and a post-state function (`spec_try_join_zombie_post`) with a lemma proving the length relationship is a clean pattern for modeling functions with side effects in a spec-only context.
- **Content-level specifications throughout.** Every state transition function specifies exact thread list contents for all paths, not just counts. This is the strongest level of specification achievable at this abstraction level.
- **Sound external_body usage.** The single `external_body` (`interrupted_resume`) has tight postconditions: PID preservation, well-formedness, sleeping/zombie passthrough, and thread count conservation. No unjustified `assume` anywhere.
- **Genuine bug discovery.** The `exit_thread()` zombie loss bug in the original source is correctly identified, well-documented in three locations, and intentionally fixed in the model.
- **Comprehensive trust boundary documentation.** Every elided function, oracle parameter, and modeling simplification is documented with rationale and caller obligations.
- **Verification passes cleanly.** 38 verification conditions, 0 errors.

## Summary

The verification of `RunningProcess` is now comprehensive and sound. All original functions are covered: 7 as verified exec implementations (`new`, `get_tid`, `schedule`, `sleep`, `exit`, `exit_thread`, `wakeup`) and 3 as spec-only models with proof lemmas (`try_join_thread`, `find_thread`, `find_thread_mut`), plus 3 accessors documented as trust boundaries (`state`, `state_mut`, `running_mut`). The specifications are strong — content-level assertions for all thread list transitions, exact branch conditions, PID preservation, and well-formedness propagation. The single external_body is well-constrained with thread count conservation. The only remaining observation is a cosmetic `>=` vs `==` in one outer ensures bound, where inner implications already establish exact equality. This verification provides high assurance for the RunningProcess state machine.
