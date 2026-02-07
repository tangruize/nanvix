# Review: mutex — Round 2 Attempt 3 (claude-opus-4.6)

## Grade: A-

## Previous Review Disposition

The previous review (claude_r2_a2) graded A- with PASSED: YES and 0 remaining issues. One non-blocking suggestion was offered (consider `Result`-based return type for `try_lock()`).

### Verification of Changes Since Previous Review

**No code changes were made.** The three module files (`mutex.rs`, `mutex.spec.rs`, `mutex.proof.rs`) are byte-identical to the version reviewed in claude_r2_a2 (commit ae1898f9). This was verified via `git diff ae1898f9 HEAD` which produced empty output.

Since the previous review already passed with zero blocking issues, and no modifications were introduced, the assessment stands unchanged.

### Previous Non-blocking Suggestion: `try_lock()` return type

**Status: Not changed — still acceptable.**

The `(bool, Tracked<Option<MutexToken>>)` return type remains. As noted in the previous review, the postconditions fully specify the bool/Option correlation, making misuse a verification-time error. This is a stylistic preference, not a correctness gap.

## Independent Re-verification of Key Properties

Since no code changed, I re-examined the module for any issues that may have been overlooked in prior rounds.

### Spec Soundness

- **`wf()` (spec line 77–78):** `self.locked == self.token_issued()`. This biconditional is the core invariant. It is established by `new()` (both false), maintained by `try_lock()` (both toggled together on success, both unchanged on failure), maintained by `lock()` (delegates to `try_lock()`), and maintained by `unlock()` (both set to false). All four exec functions ensure `self.wf()` in their postconditions. ✓
- **`spec_new_view` (spec line 87–88):** Returns `MutexView { locked: false, id: id, token_issued: false }`. Consistent with `new()` exec body at line 178. ✓
- **View implementation (spec lines 96–102):** Maps `self.locked`, `self.id@`, `self.token_issued@` to `MutexView`. Straightforward projection with ghost unwrapping. ✓

### Exec Soundness

- **`new()` (exec lines 170–178):** Postconditions match body. `wf()` holds because `locked == false == token_issued`. ✓
- **`try_lock()` (exec lines 201–226):** Success branch sets `locked = true`, `token_issued = true`, creates token with `view: self@`. Failure branch returns `(false, None)` and doesn't modify state. Postconditions are consistent with both branches. ✓
- **`lock()` (exec lines 241–257):** Preconditions guarantee `spec_is_unlocked() && wf() && !token_issued()`, which means `!locked`. So `try_lock()` takes the success branch. `opt_token.tracked_unwrap()` is safe because postcondition guarantees `result.1@.is_some()`. ✓
- **`unlock()` (exec lines 275–292):** Sets `locked = false`, `token_issued = false`. Token is consumed (moved into parameter). Postcondition `self@ == Mutex::spec_new_view(old(self)@.id)` is correct: view becomes `{locked: false, id: old_id, token_issued: false}`. ✓

### Proof Lemma Audit

All 20 lemmas have empty bodies (solver-discharged). I spot-checked three for ensures-clause correctness:

- **`lemma_mutual_exclusion` (proof lines 250–260):** Requires `wf()`, `token.view == s@`, `token.view.locked`. From `wf()`: `s.locked == s@.token_issued`. From `token.view == s@` and `token.view.locked`: `s@.locked == true`. Therefore `s.locked == true` and `s@.token_issued == true`. Ensures `s.spec_is_locked()` (= `s.locked` = true), `s@.token_issued` (true), `s@.locked` (true). ✓
- **`lemma_token_instance_isolation` (proof lines 211–222):** Requires `s1@.id != s2@.id` and `token.view == s1@`. Ensures `token.view != s2@`. Since `token.view.id == s1@.id != s2@.id`, the views differ at the `id` field. With `#[verifier::ext_equal]` on `MutexView`, structural inequality suffices. ✓
- **`lemma_no_double_unlock` (proof lines 270–278):** Requires `wf()` and `spec_is_unlocked()`. From `spec_is_unlocked()`: `!s.locked`, so `s.locked == false`. From `wf()`: `s.locked == s@.token_issued`, so `s@.token_issued == false`. Ensures `!s.locked` and `!s@.token_issued`. ✓

### Token Construction Safety (T3 re-check)

`MutexToken` has `pub ghost view: MutexView`. In Verus, `tracked struct` with `pub ghost` fields can be constructed in `proof` mode by any code that has access to the type. This means:
- Any `proof fn` in the same crate could write `let tracked t = MutexToken { view: arbitrary_view };`
- The `unlock()` precondition `token.view == old(self)@` would be satisfied if the forged view matches.

This is correctly documented as T3 at lines 110–115 of `mutex.rs`. The trust assumption is sound: as long as module consumers don't forge tokens in proof code, the protocol holds. In practice, this is a standard Verus limitation for tracked ghost state.

## New Issues

(none)

## Verification Soundness Assessment

- **Verification conditions:** 27 verified, 0 errors.
- **Escape hatches:** None (`assume`, `external_body`, `trusted`, `admit` all absent).
- **Invariant preservation:** `wf()` established by `new()`, preserved by all state transitions.
- **Token protocol:** Creation on lock, consumption on unlock, instance-bound via view identity.
- **Documentation:** 116-line module header with API mapping, divergence, trust boundaries, trust assumptions (T1–T3), verification scope.
- **Proof coverage:** 20 lemmas across definitional (13), protocol (6), and safety (5) categories, all solver-discharged.

## Summary

No changes were made since the previous review. The module remains in the same state that received A- with PASSED in round 2 attempt 2. Independent re-verification of spec soundness, exec correctness, proof lemmas, and token safety confirms the assessment. The mutex verification is sound within its stated scope: sequential state machine correctness of the lock/unlock protocol with zero escape hatches, comprehensive proof coverage, and thorough documentation. No issues remain.
