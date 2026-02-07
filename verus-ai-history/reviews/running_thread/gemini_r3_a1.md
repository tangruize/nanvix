# Review: running_thread (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### High
- **Unverified Mutable Access**: `thread_state_mut` is marked `#[verifier::external]`. This returns a `&mut ThreadState`, giving callers unrestricted access to the internal state. This bypasses all invariants (like `wf()`) and makes any code using this method part of the Trusted Computing Base (TCB).
    - **Location**: `running.rs` (exec), `thread_state_mut`
    - **Suggested Fix**: Audit all call sites of `thread_state_mut` manually. Where possible, replace raw state mutation with verified methods on `RunningThread`. For required usages, ensure they preserve the `wf()` and identity invariants documented in the comments.

### Medium
- **Missing API Member**: `join_cond()` is omitted from the verification model. Since this returns a `Condvar` used for thread joining, its omission precludes reasoning about thread termination synchronization/liveness properties involving joins.
    - **Location**: `running.rs` (exec)
    - **Suggested Fix**: Add an opaque model for `Condvar` and expose `join_cond` in the verified API to ensure the interface is complete, even if internal properties of the Condvar aren't fully modeled yet.

- **API Divergence (Mutex Release)**: `take_mutex_guard` in the verified model requires `spec_has_mutex(address)` and returns `()`. The original implementation returns `Option<MutexGuard>`, implying it can handle cases where the mutex is not found. The verified model assumes strictly correct usage (guard always exists), which is stronger than the implementation.
    - **Location**: `running.rs` (exec), `take_mutex_guard`
    - **Suggested Fix**: Confirm that no production code relies on the `None` return case of `take_mutex_guard`. If it does, the model must be updated to return an `Option` or `bool` and handle the missing-mutex case.

### Low
- **Dropped Context Return Values**: `sleep`, `schedule`, and `exit` in the implementation return a tuple including `*mut ContextInformation`. The verified model drops this return value. While this is a documented HAL boundary decision, it means the data flow of context pointers is unverified.
    - **Location**: `running.rs` (exec), `sleep`, `schedule`, `exit`
    - **Suggested Fix**: Acknowledge as a permanent limitation or introduce a `Ghost<Context>` or opaque wrapper to track the flow of these pointers if context safety becomes a verification goal.

## Positive Observations
- **Excellent Documentation**: The file includes a "Verification Model" section and "Trust Boundary" notes that clearly articulate what is and isn't verified. This is exemplary practice.
- **Cross-Module Verification Pattern**: The use of local boundary models (`SleepingThread`, `ReadyThread`, `ZombieThread`) with explicit `CROSS-MODULE-CHECK` obligations is a robust way to handle cyclic dependencies in the thread state machine without complex imports.
- **Strong Invariants**: The specifications for mutex accounting and thread identity preservation are rigorous and well-proven.
- **Clean Split**: The separation into `.rs`, `.spec.rs`, and `.proof.rs` is clean and follows the project's verification patterns effectively.

## Summary
The verification of `running_thread` is of high quality. It successfully models the core state transitions of a running thread (sleep, schedule, exit) and critical safety properties like mutex accounting. The logical separation of concerns is handled well through boundary models. The primary weakness is the `external` mutable accessor (`thread_state_mut`), which is likely a necessity for the current codebase but represents a significant hole in the verification boundary. The deviations from the original API (omitting return values, strengthening preconditions) are well-documented and justified for a verification model.
