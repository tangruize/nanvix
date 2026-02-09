# Review: sleeping_process (claude-opus-4.6)

## Grade: A

## Verification Result

18 verified, 0 errors (down from 21 — 3 trivial lemmas correctly removed).

## Previous Issue Resolutions

### Low #1: `lemma_wakeup_alarm_expired_is_wf` trivial — ✅ FIXED (removed)

Lemma removed from proof file. Verification still passes (18/18), confirming it was not needed. The trivially-true implication (`len > 0` ⟹ `len >= 1`) is now handled inline by the `assert(interrupted_ids@.len() >= 1)` in `wakeup_alarm()` at sleeping.rs:427.

### Low #2: `lemma_terminate_preserves_total_threads` tautology — ✅ FIXED (removed)

Lemma removed from proof file. It was not called anywhere in the exec code, so its removal is safe. Verification confirms no dependency.

### Low #3: `lemma_wf_and_found_implies_sleeping_positive` trivially derivable — ✅ FIXED (removed)

Lemma removed from proof file AND its call site removed from `wakeup()` (sleeping.rs:313–315, previously had `self.lemma_wf_and_found_implies_sleeping_positive(tid@)`). Verification passes without it, confirming the prover can derive `sleeping_count > 0` automatically from `wf()`.

## Detailed Verification of Changes

The diff from R1A2 to current is exactly:
1. **proof file:** Removed `lemma_terminate_preserves_total_threads` (was lines 71–81), `lemma_wf_and_found_implies_sleeping_positive` (was lines 99–106), `lemma_wakeup_alarm_expired_is_wf` (was lines 136–147).
2. **exec file:** Removed `self.lemma_wf_and_found_implies_sleeping_positive(tid@);` call from `wakeup()` proof block.
3. **spec file:** No changes.

No other modifications. No new code, no weakened specs, no added assumptions.

## New Issues Introduced

None.

## Remaining Issues

None of substance. The module is clean.

### Cosmetic (non-blocking)

- **Location:** `lemma_spec_find_sleeping_index` (proof: sleeping.proof.rs:70–77)
  **Description:** The postcondition restates the definition of `spec_seq_contains`. While trivial on paper, it serves as a witness extraction helper invoked in `wakeup()`, so its presence is justified for proof ergonomics.

## Final Assessment Checklist

| Criterion | Status | Notes |
|-----------|--------|-------|
| **COVERAGE** | ✅ Complete | All 9 original functions have verified counterparts |
| **SPECIFICATIONS** | ✅ Strong | Postconditions capture PID preservation, thread conservation, non-empty invariants, partition correctness |
| **SOUNDNESS** | ✅ Sound | No `assume`, no unjustified `external_body`. The 2 `external_body` uses (`state`, `state_mut`) have tight frame conditions |
| **EQUIVALENCE** | ✅ Faithful | Verified code models all state transitions from the original; oracle parameters are tightly constrained |
| **INVARIANTS** | ✅ Sufficient | `wf()` captures count consistency, non-empty sleeping list, no-duplicates, and cross-list disjointness |
| **PROPERTIES** | ✅ Proven | PID immutability, thread conservation across terminate/wakeup/wakeup_alarm, partition correctness, non-empty guarantees |
| **SPLIT QUALITY** | ✅ Clean | Spec: only spec functions + views. Proof: only lemmas. Exec: only implementations with contracts |

## Positive Observations

- **All 10 issues across 3 review rounds resolved.** The prover systematically addressed every issue from Critical through Low priority.
- **Proof file is now lean and purposeful.** 18 verified items with no tautological filler. Every remaining lemma serves a clear purpose (construction wf, frame preservation, index witness extraction, remove-at properties, find_thread consistency, view equality, boundary type wf).
- **Strong oracle constraints on `wakeup_alarm`.** The combination of subset membership + no-duplicates + disjointness + length conservation achieves multiset partition equality — the gold standard for partition verification.
- **`wf()` fully models Rust ownership semantics.** `spec_no_duplicates` + `spec_seqs_disjoint` capture what the Rust borrow checker enforces, enabling precise reasoning in the ghost domain.
- **No unsound shortcuts.** Zero `assume` statements, zero `#[verifier::external]` on core logic, zero `admit`. The only `external_body` annotations are on accessor methods with proper frame conditions.
- **Documentation is thorough and accurate.** Module headers, trust boundary documentation, oracle parameter explanations, and per-function doc comments are all consistent with the actual verification.

## Summary

The verification of `SleepingProcess` is complete and sound. All 9 functions from the original source are covered with strong contracts. The `wf()` invariant captures the essential structural properties (non-empty sleeping list, count consistency, thread ID uniqueness, cross-list disjointness). State transitions are proven to preserve PID, conserve threads, and maintain non-empty guarantees on output types. Oracle parameters for `wakeup` and `wakeup_alarm` are tightly constrained to prevent fabrication. The proof file is clean with no tautological clutter. No remaining issues.
