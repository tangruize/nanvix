# Re-Review: mutex (claude-opus-4.6)

## Grade: A

## Previous Issue Disposition

### High Issues

- **`lock()` sequential limitation** — **Fixed.** The `lock()` doc comment (line 225–229) now explicitly states: "this is a blocking operation ... can be called on an already-locked mutex. In the sequential model, the `spec_is_unlocked()` precondition guarantees `try_lock()` succeeds on the first attempt, so contended locking (the primary concurrent use case) is outside the verified model's coverage." This is clear and prominent. The aspirational suggestion to add a protocol-level multi-thread lemma was not done, but this is reasonable — the `&mut self` model fundamentally cannot express interleaved thread access, so such a lemma would require a different modeling approach entirely.

- **`try_lock()` `&mut self` limitation** — **Fixed.** The `try_lock()` doc comment (lines 182–185) now contains an explicit **Note** paragraph: "The original takes `&self` with atomic interior mutability. The verified version takes `&mut self` (exclusive reference), so this verification covers state machine transitions only, not the concurrent correctness that `compare_exchange` provides." This directly addresses the concern.

### Medium Issues

- **`pub` fields comment** — **Fixed.** The struct's `# Representation` section (lines 134–136) now reads: "Per Nanvix coding standards, struct fields should be private with getter/setter access; this is an exception due to Verus tooling constraints." This satisfies the request. The suggestion to investigate `spec(checked)` accessor patterns was exploratory and not required.

- **Missing `fmt::Debug` in API mapping** — **Fixed.** Line 66 adds `fmt::Debug for MutexGuard | (not modeled) | Display-only, no state mutation.` to the table. The `notify_first()` error path omission is also now documented in the API Divergence section (lines 84–87).

- **Stronger proof lemmas** — **Partially fixed.** See detailed analysis under "New/Remaining Issues" below. `lemma_no_double_unlock` is a genuine, well-named addition. `lemma_mutual_exclusion` has a naming/documentation problem.

- **`wf()` local invariant documentation** — **Fixed.** The `wf()` doc comment in `mutex.spec.rs` (lines 73–76) now contains a clear **Note** explaining it is a local per-mutex invariant and does not capture global token uniqueness.

### Low Issues

- **Ghost ID uniqueness** — **Correctly left as-is.** Already documented as trust assumption T1. No action was required.

- **`unlock()` error path** — **Fixed.** Documented in API Divergence section (lines 84–87).

- **Tautological lemma replacement** — **Fixed.** The original `lemma_lock_token_snapshot_is_locked` (which asserted `token.view == MutexView { locked: true, ... }` given `token.view.locked` — a structural identity tautology) was replaced with `lemma_locked_wf_implies_token_state` (lines 107–118), which combines `wf()` and `spec_is_locked()` to derive the concrete view `MutexView { locked: true, id: s@.id, token_issued: true }`. This is a meaningful improvement — it derives `token_issued: true` from `wf()`, which the original did not.

## New/Remaining Issues

### Medium

- **Location:** `mutex.proof.rs`, `lemma_mutual_exclusion`, lines 240–260
  - **Description:** The lemma name and documentation overclaim relative to what the `ensures` clause proves. The doc comment states: "Since token_issued is a single boolean, at most one token can be outstanding per well-formed mutex instance." However, the lemma proves only: given `s.wf()`, `token.view == s@`, and `token.view.locked`, then `s.spec_is_locked()` and `s@.token_issued`. This is a conditional implication about a *single* token — it does not prove that two tokens cannot coexist. The actual mutual exclusion guarantee comes from the exec-level linearity of `try_lock()`/`lock()` (which produce exactly one token and set `token_issued = true`, blocking further issuance via `wf()`), not from this lemma alone.

    Furthermore, the ensures clause is definitional unfolding: `token.view == s@` and `token.view.locked` directly give `s@.locked == true`, then `wf()` gives `s@.token_issued == true`. This is the same reasoning chain as `lemma_locked_wf_implies_token_state` (lines 107–118) with an extra indirection through the token parameter.

  - **Suggested Fix:** Rename to `lemma_valid_token_implies_locked` and adjust the doc comment to avoid claiming "mutual exclusion" or "at most one token." Alternatively, to genuinely prove mutual exclusion, add a lemma showing that given `s.wf()` and `s.spec_is_locked()` (i.e., `token_issued == true`), the `try_lock()` postconditions guarantee `result.0 == false` (lock acquisition fails), which is the actual mechanism preventing a second token from being issued.

### Low

- **Location:** `mutex.proof.rs`, `lemma_no_double_unlock` (lines 262–278) vs `lemma_wf_unlocked_no_token` (lines 96–105)
  - **Description:** `lemma_no_double_unlock` has identical preconditions to the existing `lemma_wf_unlocked_no_token` (`s.wf(), s.spec_is_unlocked()`) and its ensures clause (`!s.locked, !s@.token_issued`) is a strict superset — adding only `!s.locked`, which is trivially `s.spec_is_unlocked()` restated. While the name "no_double_unlock" is more descriptive of the *intent*, the two lemmas are near-duplicates. This is minor noise, not a correctness issue.
  - **Suggested Fix:** Consider merging or adding a comment linking the two to explain why both exist.

## Verification Status

- **26 verified, 0 errors** (up from 24 in the original submission).
- **No `assume`, `external_body`, or `trusted` annotations.** The verification is fully automated with no trust gaps.
- **No regressions.** All original verification conditions continue to pass.

## Positive Observations

- **Documentation is now excellent.** Every concern about missing documentation was addressed with clear, accurate text. The `lock()`, `try_lock()`, `wf()`, API mapping, and API divergence sections are all improved.
- **No overclaiming.** The documentation is honest about what the sequential model proves and what it cannot. The new `wf()` note about local vs. global invariants is particularly good.
- **The replacement lemma is better.** `lemma_locked_wf_implies_token_state` is strictly more informative than the tautology it replaced.
- **`lemma_no_double_unlock` has good intent.** Even though it's near-duplicate, explicitly naming the no-double-unlock property makes the proof artifact more readable as a specification document.
- **Clean, surgical changes.** The prover made minimal, focused edits — no unnecessary refactoring or code churn.

## Summary

The prover addressed 9 of 10 actionable items from the original review. Documentation improvements are thorough and accurate across all three files. The tautological lemma was replaced with a genuinely better one. The only substantive gap is the `lemma_mutual_exclusion` naming — it claims to prove mutual exclusion but actually proves a conditional single-token implication that is definitional unfolding. This is a documentation/naming issue, not a soundness issue; the underlying verification is correct and complete.

The module now has 26 fully verified conditions with no trust gaps, comprehensive documentation of limitations and divergences, and a clean spec/proof/exec separation. This is a well-executed verification of the mutex state machine protocol.
