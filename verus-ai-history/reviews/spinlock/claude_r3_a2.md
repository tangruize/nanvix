# Re-Review: spinlock (claude-opus-4.6)

## Grade: A-

## Previous Issue Resolution

### High Issues

- **`try_lock()` preconditions (failure path dead code)** — **FIXED ✓**
  - **Previous:** Precondition `old(self).wf() && !old(self).token_issued()` forced `!old(self).locked`, making the `else` branch unreachable.
  - **Current:** Precondition weakened to just `old(self).wf()` (line 173 of exec). Now when `wf()` holds: (a) `locked=false` → success path; (b) `locked=true` → failure path (since `wf()` gives `locked == token_issued`, a locked lock has `token_issued=true`). Both paths are genuinely reachable. The postconditions `!result.0 ==> self@ == old(self)@` and `!result.0 ==> result.1@.is_none()` are now non-vacuously verified. The doc comment accurately claims "Both success and failure paths are non-vacuously verified" (line 163-165). **This is the most important fix in this revision.**

- **`lock()` preconditions (requires `spec_is_unlocked()`)** — **Addressed as suggested ✓**
  - **Previous suggestion:** Add documentation or a proof lemma justifying why the precondition is necessary.
  - **Current:** Doc comment at lines 204-206 clearly explains the deadlock prevention rationale. New lemma `lemma_lock_precondition_prevents_deadlock` (proof line 246-254) formalizes the argument: a locked, well-formed spinlock has a token outstanding, so `spec_is_unlocked()` cannot hold — making `lock()` unreachable on a locked spinlock, which is correct because it would deadlock in the sequential model. The precondition remains but is now well-justified.

### Medium Issues

- **Trivial proof lemmas** — **Partially addressed**
  - **Improvement:** Lemmas are now organized into two clearly labeled sections: "Definitional Properties" (lines 10-95) and "Substantive Protocol Properties" (lines 99-255). The section header at line 15 honestly describes the definitional lemmas as "definition-unfolding properties that serve as executable documentation and regression tests for spec changes." This is a significant organizational improvement.
  - **Remaining concern:** Some lemmas in the "Substantive" section are still definitional (see New Issues below). However, this is a labeling issue, not a soundness issue.

- **`pub` struct fields** — **Unchanged, acceptable.** Verus limitation, adequately documented at lines 107-111.

- **Ghost `id` uniqueness** — **Unchanged, acceptable.** Trust assumption T1 at lines 78-82 is thoroughly documented.

### Low Issues

- **`SpinlockGuard` not modeled as a type** — **Unchanged, acknowledged limitation.**
- **`new()` signature divergence** — **No fix needed.** Already documented.
- **`&self` vs `&mut self`** — **Unchanged.** Doc comment at lines 53-58 clearly explains.

## New Issues

### Low

- **Location:** Proof lemma categorization (proof: `spinlock.proof.rs`, lines 99-255)
  - **Description:** Four lemmas under "Substantive Protocol Properties" are actually definition-unfolding properties with empty proof bodies. Specifically:
    - `lemma_try_lock_contended_fails` (line 227): `wf() ∧ spec_is_locked()` ⟹ `locked ∧ token_issued` — follows directly from `spec_is_locked() ≡ locked` and `wf() ≡ locked == token_issued`.
    - `lemma_lock_precondition_prevents_deadlock` (line 246): same preconditions, same pattern — `token_issued ∧ !spec_is_unlocked()` follows directly from definitions.
    - `lemma_wf_unlocked_no_token` (line 210): `wf() ∧ spec_is_unlocked()` ⟹ `!token_issued` — single definition unfolding.
    - `lemma_lock_token_snapshot_is_locked` (line 180): requires `token.view.locked`, concludes the view equals itself with `locked: true` — tautological decomposition.
  - The truly substantive lemmas (multi-step reasoning) are: `lemma_lock_unlock_roundtrip`, `lemma_token_instance_isolation`, `lemma_unlocked_eq_new_view`, `lemma_new_then_try_lock_succeeds`, and `lemma_lock_token_valid_for_unlock`.
  - **Impact:** Low — this is a labeling issue only. The lemmas are not incorrect and may serve as documentation. The section header's claim of "non-trivial protocol properties that require reasoning across multiple spec definitions" is overstated for these four.
  - **Suggested Fix:** Either move these four to the "Definitional Properties" section, or soften the section header to "Protocol Properties" (dropping "non-trivial" / "require reasoning across multiple spec definitions").

- **Location:** `try_lock()` postcondition `self.locked` (exec: line 176)
  - **Description:** `try_lock()` guarantees `self.locked` unconditionally in the postcondition — meaning the lock is always in the locked state after the call, regardless of success or failure. This is correct (success: lock acquired; failure: was already locked, no mutation) but may surprise callers who expect a failed `try_lock` to leave state completely unchanged. The postcondition `!result.0 ==> self@ == old(self)@` (line 178) already captures state preservation on failure, which implies `self.locked` when it was already locked. So `self.locked` is redundant with the combination of line 175 (`result.0 == !old(self).locked`) and line 178. Not a bug, but the redundancy could be confusing.
  - **Suggested Fix:** Consider adding a brief comment on line 176 noting this is an unconditional postcondition that follows from both paths. Alternatively, remove it as it's derivable from the other postconditions.

## Positive Observations

- **No `assume`, `external_body`, or `trusted` annotations.** Fully verified with zero trust holes in core logic. This remains the gold standard.
- **The `try_lock()` fix is the standout improvement.** Weakening the precondition to just `wf()` makes both success and failure paths genuinely exercised. This transforms `try_lock` from a trivially-succeeds wrapper to a genuine conditional operation, which is exactly what the previous review requested.
- **Excellent documentation quality.** The module-level doc comment is comprehensive, transparent about limitations, and accurately describes the verification scope. The trust boundaries, assumptions, and API divergences are clearly delineated. The `lock()` doc comment's deadlock prevention rationale (lines 204-206) is a particularly good addition.
- **Sound `lock()` delegation.** With the weakened `try_lock()`, `lock()`'s delegation is still sound: its preconditions (`spec_is_unlocked() ∧ wf() ∧ !token_issued()`) imply `!locked`, so `try_lock()` succeeds, and `tracked_unwrap()` on the `Some(token)` is safe. This is clean verified code, not an `external_body` escape.
- **Well-formed invariant preservation.** The biconditional `wf()` predicate (`locked == token_issued`) is preserved across all transitions: `new()` ⟹ `(false, false)`, `try_lock()` success ⟹ `(true, true)`, `try_lock()` failure ⟹ unchanged, `unlock()` ⟹ `(false, false)`. All verified without assumptions.
- **Clean organizational structure.** The three-file split (exec/spec/proof) with the `include!` mechanism is maintained cleanly. The proof file's two-section organization with honest section headers is a good improvement.
- **Token-based RAII modeling remains sound.** Linear `LockToken` with view-based identity binding prevents cross-instance misuse, verified by `lemma_token_instance_isolation`.

## Summary

This revision makes the single most impactful improvement available: weakening `try_lock()`'s precondition to exercise both success and failure paths non-vacuously. This was the primary actionable recommendation from the previous review, and it was correctly implemented — the postconditions on both paths are now genuinely verified, not vacuously true. The `lock()` delegation remains sound under its stronger preconditions.

The documentation improvements are thorough and honest. The `lock()` deadlock prevention rationale and the proof lemma `lemma_lock_precondition_prevents_deadlock` directly address the second high-priority issue from the previous review.

The remaining issues are all low-priority: lemma categorization (labeling, not soundness), a redundant postcondition, and the inherent limitations of the sequential `&mut self` model (pub fields, ghost ID uniqueness, no lifetime modeling). The verification is sound within its declared scope and transparent about its boundaries.

**Upgrade from B+ to A-:** The `try_lock` fix addresses the most significant verification weakness. The remaining gap to A is the lemma categorization and the fact that a majority of proof lemmas (13 of 17) remain definitional, limiting the depth of verified properties beyond the core protocol.
