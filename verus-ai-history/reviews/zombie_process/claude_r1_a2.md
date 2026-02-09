# Review: zombie_process (claude-opus-4.6) — Round 2

## Grade: A

## Previous Issues Disposition

### High #1: `state_mut()` PID immutability trust assumption
**Status: ADDRESSED (documentation + architectural justification)**

The prover added extensive documentation (zombie.rs lines 44–53, 188–196) explaining that PID immutability is enforced architecturally: `ProcessState` does not expose a public setter for `pid`. I independently verified this claim:
- `ProcessState.pid` is a **private** field (no `pub` keyword in the struct definition).
- The only place `pid` is set is in the `new()` constructor.
- No setter method for `pid` exists anywhere in `ProcessState`.
- The only `&mut` accessor is `vmem_mut()`, which returns `&mut Vmem` — not transitive to `pid`.
- No code in `src/kernel/` directly assigns to a ProcessState's `pid` field.

The trust assumption is architecturally sound. The documentation now clearly states the conditions under which it would become unsound (if a PID setter is ever added). **Accepted.**

### High #2: `find_thread()`/`find_thread_mut()` unverified search
**Status: SUBSTANTIALLY IMPROVED**

Three meaningful changes were made:
1. **`lemma_ghost_search_correctness`** (proof lines 163–198): A new, non-trivial proof that verifies the ghost-level search logic over `Seq<int>` — forward direction (any matching index → found), backward direction (found → exists matching index), and completeness (no match → None). This uses explicit witness reasoning and is genuinely useful.
2. **Renamed** `lemma_find_thread_refinement_assumption` → `lemma_find_thread_completeness` with corrected documentation stating it's a spec-level property, not a refinement proof.
3. **Documentation** throughout now says "UNVERIFIED SEARCH" and flags the integration obligation as "unproven."

The ghost-level search proof adds real assurance that the *specification* correctly models membership. The remaining gap — that the executable `iter().find()` matches this spec — is an inherent limitation honestly documented. **Accepted.**

### Medium #1: `state()` abstraction boundary
**Status: ADDRESSED** — Documented in zombie.rs lines 54–60 and function doc (lines 163–170). **Accepted.**

### Medium #2: `wf()` missing thread ID validity
**Status: ADDRESSED** — Documented in zombie.spec.rs lines 127–131 as outside scope, deferring to thread module verification. **Accepted.**

### Medium #3: `new()` extra `zombie_count` parameter
**Status: ADDRESSED** — Documented in zombie.rs lines 123–126. **Accepted.**

### Medium #4: `bury()` ownership transfer not captured
**Status: ADDRESSED** — Documented in zombie.rs lines 72–79 and function doc (lines 212–219). **Accepted.**

### Low #1: `lemma_find_thread_refinement_assumption` misleading name
**Status: FIXED** — Renamed to `lemma_find_thread_completeness` with corrected documentation (proof line 203). **Accepted.**

### Low #2: `ZombieProcessView` unused
**Status: FIXED** — `bury()` postconditions now use `self@.zombie_thread_ids`, `self@.pid`, `self@.status` (zombie.rs lines 228–230), and `lemma_bury_matches_view` (proof lines 95–105) uses the View type. **Accepted.**

### Low #3: `spec_seq_contains` dead code
**Status: FIXED** — `spec_has_zombie_thread` now delegates to `spec_seq_contains` (zombie.spec.rs line 97), and `lemma_ghost_search_correctness` references it. Moved before `spec_has_zombie_thread` for proper definition order. **Accepted.**

## New Issues Found

### Medium

- **Location:** `lemma_ghost_search_correctness` backward direction (proof — zombie.proof.rs:189–190)
  **Description:** The backward direction ("if spec finds it, there exists a valid index") has only a comment (`// This follows directly from the definition of spec_seq_contains`) but no explicit proof body or `assert` statement. While Verus accepts this (the SMT solver can discharge it from the existential definition), this is inconsistent with the forward and completeness directions which both have explicit proof steps. For a proof meant to demonstrate search correctness, leaving the most important direction to implicit SMT solving weakens readability and maintainability.
  **Suggested Fix:** Add an explicit assertion, e.g., `assert(self.spec_has_zombie_thread(tid));` for symmetry with the other directions.

### Low

- **Location:** `lemma_bury_preserves_pid` and `lemma_bury_preserves_status` (proof — zombie.proof.rs:108–118)
  **Description:** These two lemmas are now redundant with the more comprehensive `lemma_bury_matches_view` (which proves all three fields match the View, plus length invariants). They prove trivial tautologies (`self.spec_pid() == self.pid@`) that follow directly from spec function definitions. They add no verification value beyond what `lemma_bury_matches_view` already provides.
  **Suggested Fix:** Remove these two lemmas or mark them as convenience aliases for downstream consumers. Not urgent, but they add unnecessary proof surface area.

## Positive Observations

- **All 17 verification conditions pass** (up from 16, confirming the new proof is valid).
- **`lemma_ghost_search_correctness` is genuine.** It uses explicit witness reasoning in the forward direction and contrapositive negation in the completeness direction — this is real proof work, not boilerplate.
- **Trust boundary documentation is now exemplary.** Each trust assumption is labeled ("TRUST ASSUMPTION", "UNVERIFIED SEARCH"), conditions for unsoundness are stated, and integration obligations are explicitly flagged as "unproven." This is exactly how design-level verification trust boundaries should be documented.
- **The `spec_seq_contains` refactoring is clean.** Factoring `spec_has_zombie_thread` through `spec_seq_contains` creates a reusable abstraction and eliminates the previous code duplication.
- **View type is now actively used.** `bury()` postconditions reference `self@` and `lemma_bury_matches_view` connects the raw fields to the abstract View, improving specification clarity.
- **The PID immutability trust assumption is validated by architecture.** Independent verification confirms `ProcessState.pid` is private with no setter — the strongest form of "documentation-level" evidence possible short of verifying ProcessState itself.

## Summary

The prover has thoroughly and honestly addressed all issues from Round 1. Every High and Medium issue was either fixed (structural code changes) or addressed with detailed, accurate documentation of trust boundaries. All three Low issues received actual code fixes (rename, refactor, wire up unused abstractions). The new `lemma_ghost_search_correctness` proof adds genuine verification value, proving the ghost-level search logic is sound with explicit forward/backward/completeness reasoning.

The two remaining issues are minor: one stylistic inconsistency in proof structure, and two redundant lemmas. Neither affects soundness or verification coverage.

This is a well-executed design-level verification with clear, honest trust boundaries. The module is ready for integration.
