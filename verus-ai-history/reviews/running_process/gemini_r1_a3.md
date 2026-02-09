# Review: running_process (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None. The previous logic bug in `exit_thread` (original source) remains fixed.

### Medium
- **Soundness / Oracle Parameter in `wakeup`** (Remaining): The `wakeup` function continues to use a `found: bool` oracle parameter. As noted in previous reviews, this relies on a trusted precondition (`found == spec_seq_contains(...)`) rather than verifying the execution-level search logic (`remove_if`).
- **Trust Boundary / Mutability** (Remaining): `state_mut()` and `running_mut()` continue to return `Ghost<int>`, deferring the preservation of structural invariants to unverified callers.

### Low
- **Elided Types** (Remaining): Contexts and Condvars are elided, which is verified as acceptable for this scope.

## Positive Observations
- **Bug Fix Verified**: Validated that the original source code in `src/kernel/src/pm/process/state/running.rs` correctly passes `Some(zombie_threads)` instead of `self.zombie.take()` in `exit_thread`, ensuring zombie threads are not lost.
- **Specification Quality**: The specifications accurately model the complex state transitions of the process lifecycle.
- **Stability**: The verification continues to pass successfully.

## Summary
The verification status is stable. The critical bug in the original kernel source code has been fixed and verified. The remaining issues regarding the oracle parameter in `wakeup` and the trust boundaries for mutable accessors have not been changed, but they are documented and accepted as trade-offs for this specific verification scope. The module is sound within these stated assumptions.
