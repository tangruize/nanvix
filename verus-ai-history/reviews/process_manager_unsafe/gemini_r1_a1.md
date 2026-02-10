# Review: process_manager_unsafe (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Loop Verification in `join_thread`**: The original `join_thread` function contains a loop that retries upon waking up from a condition variable. The verified model splits this into three separate functions (`join_thread_harvest`, `join_thread_wait`, `join_thread_error`) that verify the safety of each path but do not formally verify the loop structure or termination. While this is acceptable for safety verification (invariant preservation), it means the control flow of the loop itself is not machine-checked.

## Positive Observations
- **Accurate Modeling of Stale Reads**: The verification of `switch()` correctly captures the subtle interaction between the inner state mutation and the atomic variables. Specifically, it verifies that `switch` uses the *new* `inner` state but compares `next_pid` against the *old* `current_pid` (from the atomic) to detect process switches. This is a critical correctness property of the context switching logic.
- **Comprehensive Trust Boundary Documentation**: The module includes excellent documentation of all trust boundaries (T5-T13), clearly explaining what is verified, what is deferred to other modules (like thread-PID membership), and what is outside the scope of Verus (like divergence).
- **Unified Quantum Management**: The `giveup()` model correctly unifies the "decrement quantum" and "context switch" paths, ensuring that the quantum invariant `1 <= remaining_quantum <= scheduler_freq` is always preserved.
- **Clean Separation**: The code is well-organized into exec, spec, and proof files, making it easy to read and understand the verification logic.

## Summary
The verification of `process_manager_unsafe` is of high quality. It successfully models the complex interaction between the `ProcessManagerInner` state machine and the global atomic variables used for low-level context switching. The model correctly identifies and proves the key safety invariants, particularly the consistency between the inner running PID and the global `CURRENT_PID` atomic. The limitations (such as not modeling the `! ` return type of exit functions) are well-understood and documented. The verification provides strong assurance that the unsafe wrapper correctly manages the global state during context switches.
