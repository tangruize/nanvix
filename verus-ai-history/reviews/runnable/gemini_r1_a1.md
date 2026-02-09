# Review: runnable (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Missing Exec Functions**: `earliest_admission_time`, `find_thread`, and `find_thread_mut` are modeled as spec-only functions (`spec_earliest_admission_time`, `spec_find_thread`). While this is sufficient for verifying the internal logic of `run` and `wakeup`, it means these public methods are not available for other verified modules to call at the exec level.
    - **Fix**: If these functions are needed by verified clients, implement exec wrappers that invoke the spec functions (for `earliest_admission_time`) or return a simplified result (e.g., an enum for `find_thread` instead of references).

## Positive Observations
- **Strong Specifications**: The specifications precisely capture the behavior of thread list manipulations (push, remove, append) and ensure no threads are lost during transitions.
- **Oracle Removal**: The use of exec-level counters (`interrupted_count`, `sleeping_count`) linked to ghost sequence lengths in the `wf` predicate is a great technique. It allows `terminate()` to be verified without requiring an oracle parameter for the branch decision, making the model closer to the implementation.
- **Constructive Proofs**: The replacement of the linear search loop in `run()` with a constructive proof of minimum existence (`lemma_earliest_ready_index_bounds`) is an elegant abstraction that simplifies the exec code while rigorously proving the property.
- **Clear Documentation**: The documentation clearly explains the abstraction model, trust boundaries (HAL types), and the handling of ownership semantics via trust assumptions.

## Summary
The verification of `RunnableProcess` is of high quality. It covers all state transitions (`run`, `terminate`, `wakeup`, `add_thread`) with strong postconditions that guarantee thread accounting and correct state updates. The abstraction choices (ghost sequences, eliding HAL types) are well-justified and documented. The verification successfully proves that the process scheduling logic—including the critical earliest-deadline-first selection in `run()`—is correct.
