# Review: fence (claude-opus-4.6) — Round 3

## Grade: A-

## Previous Issues — Disposition

### From Round 1 (retained as documented limitations)

- **`wait()` requires `self.spec_is_satisfied()` precondition** — Inherent sequential modeling limitation. Documented transparently at `fence.rs` lines 76–84 as "Key verification gap." **Accepted.**

- **`signal(&mut self)` vs original `&self`** — Inherent sequential modeling limitation. Documented at `fence.rs` lines 85–89. **Accepted.**

- **`pub` fields** — Verus limitation. Documented at `fence.rs` line 121. **Accepted.**

- **`const fn` not supported** — Verus limitation. Documented at `fence.rs` lines 138–139. **Accepted.**

### From Round 2

- **Stale `lemma_total_signals_satisfies` cross-reference in `fence.proof.rs` (FIXED)**
  - **Status:** Verified fixed. The diff between commits `ccb60e5c` and `05e0c5c8` shows exactly one line changed: line 238 updated from `lemma_total_signals_satisfies` to `lemma_equality_implies_satisfaction`. Confirmed by grep: zero remaining occurrences of the old name across all three module files. **Properly fixed.**

## New Issues Found

_None._

## Verification

All 24 verification conditions pass. No `assume`, `external_body`, `trusted`, or `axiom` annotations present. No soundness escape hatches.

## Positive Observations

- **All actionable issues resolved.** Across three review rounds, every fixable issue has been addressed. The remaining items are inherent modeling limitations, all well-documented.

- **Exemplary documentation quality.** The module-level documentation in `fence.rs` is thorough, transparent, and accurate. Trust boundaries, API divergences, verification scope, and modeling decisions are all explicitly documented with precise references to the original codebase.

- **No soundness escape hatches.** The entire module verifies without any `assume`, `external_body`, `trusted`, or `axiom` annotations.

- **Clean spec/proof/exec separation.** The three-file split follows the project convention cleanly.

- **Well-organized proof structure.** The proof file has three clearly delineated sections (Definitional Properties, Protocol Properties, Concurrency Properties) with explanatory headers that set appropriate expectations for each section's depth.

- **Comprehensive `signal()` postconditions** covering count increment, total preservation, wf preservation, remaining decrement, and satisfaction on last signal.

## Summary

This is a mature, well-documented verification module after three rounds of review. All fixable issues have been resolved. The remaining items are inherent limitations of the sequential modeling approach (`wait()` precondition, `signal(&mut self)`), which are documented with exceptional transparency. The verification is sound within its stated scope: sequential state-machine correctness of the fence protocol. No new issues found in this round.
