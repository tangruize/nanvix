# Review: running_process (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **(FIXED)** **Equivalence / Bug in Original Source**: The logic bug in `exit_thread` (dropping zombies in the interrupted branch) has been fixed in the original source code (`src/kernel/src/pm/process/state/running.rs`). The verification model now correctly matches the corrected implementation.

### Medium
- **Soundness / Oracle Parameter in `wakeup`** (Remaining): The `wakeup` function still uses a `found: bool` oracle parameter. The actual search logic (`NonEmptyVecDeque::remove_if`) inside `wakeup` is not verified; instead, it is assumed that the `found` parameter correctly reflects whether the thread exists in the ghost sequence (`found == spec_seq_contains(...)`). This leaves a gap where the exec-level search could diverge from the spec.
- **Trust Boundary / Mutability** (Remaining): `state_mut()` and `running_mut()` still return `Ghost<int>`, relying on unverified callers to preserve structural invariants of the `ProcessState` and `RunningThread`.

### Low
- **Elided Types** (Remaining): Contexts and Condvars are still elided, which is acceptable for this level of abstraction.

## Positive Observations
- **Successful Bug Fix**: The verification process successfully drove a fix in the kernel source code, demonstrating high value.
- **Passing Verification**: The updated model verifies successfully with the corrected code.
- **Documentation**: The divergence is no longer a "known divergence" but a documented "bug fix", which is accurate.

## Summary
The verification of `running_process` is now in a very strong state. The critical discrepancy between the model and the code was resolved by fixing the code. The remaining issues (oracle for search, elided references) are acceptable trade-offs for a modular verification approach, provided that the caller/implementation of those specific features is verified separately or trusted. The grade is raised to A- to reflect the high value of the bug fix and the correctness of the core state machine transitions.
