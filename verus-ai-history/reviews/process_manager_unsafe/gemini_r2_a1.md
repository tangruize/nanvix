# Review: process_manager_unsafe (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- None.

## Positive Observations
- **Comprehensive Coverage**: All functions in `unsafe.rs` including `init`, singleton accessors (`get`/`get_mut`), and complex control flow functions (`exit`, `sleep`, `giveup`, `switch`) are fully modeled and verified.
- **Accurate Context Switch Modeling**: The `switch` verification model correctly captures the subtle "stale read" behavior of the original code, where `next_pid` is compared against the *old* `CURRENT_PID` atomic (before inner mutation) to detect process switches. This ensures faithful semantic equivalence.
- **Machine-Checked Divergence**: The use of the `ghost_diverged` flag to model non-returning functions (`exit`, `exit_thread`) is an excellent technique. It allows the `wf()` invariant to enforce that no operations can occur after a process terminates, providing a machine-checked guarantee of divergence.
- **Unified Logic**: The `giveup` model correctly unifies the "quantum decrement" (no switch) and "quantum expired" (switch) paths, ensuring both are verified under a single contract.
- **Explicit Trust Boundaries**: The documentation clearly articulates the trust boundaries (T5-T13), particularly regarding cross-module invariants like TID-to-PID membership and per-thread message queues, which are properly delegated to their respective modules.

## Summary
The verification of `process_manager_unsafe` is of high quality. It successfully bridges the gap between the unsafe global state management and the verified inner process manager logic. The specifications are strong, sound (no `assume`/`external_body`), and accurately reflect the implementation's behavior, including low-level details like atomic stale reads during context switches. The split between execution code, specifications, and proofs is clean and maintainable.
