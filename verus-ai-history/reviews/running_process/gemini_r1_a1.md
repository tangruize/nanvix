# Review: running_process (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### High
- **Equivalence / Bug in Original Source**: In `exit_thread`, the verification model diverges from the original code to fix a bug. In the original `src/kernel/src/pm/process/state/running.rs` (lines 281-287), `self.zombie.take()` is called in the interrupted branch, but `self.zombie` was already consumed at line 261. This causes the just-exited thread (and any previous zombies) to be dropped/lost instead of being passed to the `InterruptedProcess`. The verified model correctly passes `new_zombie_ids`.
    - **Suggested Fix**: Fix the bug in the original source code to match the verified model (pass the correct zombie list to `InterruptedProcess::from_sleeping`).

### Medium
- **Soundness / Oracle Parameter**: The `wakeup` function uses a `found: bool` parameter as a trusted oracle for whether the thread exists in the sleeping list. The precondition `found == spec_seq_contains(...)` cannot be checked at runtime and places a proof obligation on callers that may be difficult to discharge if the caller is not verified or if the search logic (using `NonEmptyVecDeque::remove_if`) is not modeled.
    - **Suggested Fix**: Model the search/remove logic in `wakeup` explicitly, or provide a verified wrapper that performs the search and proves the boolean is correct.

- **Trust Boundary / Mutability**: `state_mut()` and `running_mut()` return `Ghost<int>` (simulating references) and allow arbitrary mutation of the underlying `ProcessState` and `RunningThread`. The verification relies on the caller to preserve structural invariants (PID immutability, thread list integrity). This assumes correct usage by unverified callers.
    - **Suggested Fix**: Ensure that `ProcessState` and `RunningThread` are verified in the future to discharge these assumptions.

### Low
- **Elided Types**: `ContextInformation`, `Condvar`, and `SystemTime` (alarm) are elided. This is acceptable for the current scope (process state machine verification) but limits the verification of context switching and timing properties.

## Positive Observations
- **Bug Discovery**: The verification effort successfully identified a logic bug in `exit_thread` regarding zombie list handling.
- **Strong Specifications**: The `RunningProcessView` and associated spec functions capture the complete state of the process, including the complex "optional non-empty list" semantics mapped to sequences.
- **Coverage**: All public and private methods are covered, including complex transitions in `sleep` and `exit`.
- **Invariants**: Well-formedness invariants (`wf`) correctly track the correspondence between execution-level counters and ghost sequence lengths.

## Summary
The verification of `running_process` is high quality and valuable, specifically because it identified a logic bug in the original source code (`exit_thread`). The specifications are detailed and cover the complex state transitions well. The primary valid concern is the divergence in `exit_thread` (which should be fixed in the original code) and the reliance on an oracle for `wakeup`. Once the original code is patched to match the model, the equivalence score will improve.
