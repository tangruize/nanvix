# Review: running_thread (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### Medium
- **Missing `join_cond` function**
  - **Location**: `verus/split/kernel/pm/thread/running.rs` (exec)
  - **Description**: The original `RunningThread` exposes `join_cond()`, but it is omitted in the verified model. This breaks API equivalence. Even if `Condvar` is opaque, the method should be present to maintain the interface.
  - **Suggested Fix**: Add `join_cond()` to the verified `RunningThread`. If `Condvar` is not modeled, mark it `#[verifier::external]` or introduce a boundary model for `Condvar`.

- **API Mismatch in `take_mutex_guard`**
  - **Location**: `verus/split/kernel/pm/thread/running.rs`, function `take_mutex_guard`
  - **Description**: The original returns `Option<MutexGuard>`, allowing the caller to handle the case where the mutex is not held. The verified version returns `()` and requires `spec_has_mutex` as a precondition, effectively assuming the lock is always held. This hides potential bugs where code attempts to unlock a mutex it doesn't own.
  - **Suggested Fix**: Change return type to `Option<()>` (since `MutexGuard` is elided) and verify both the `Some` and `None` paths, ensuring the spec reflects the fallible nature of the operation.

### Low
- **Renamed `put_mutex_guard`**
  - **Location**: `verus/split/kernel/pm/thread/running.rs`
  - **Description**: The original method is named `put_mutex_guard`. The verified version is named `store_mutex_guard`.
  - **Suggested Fix**: Rename the verified function to `put_mutex_guard` to match the original source.

- **Unverified Context Switching**
  - **Location**: `sleep`, `schedule`, `exit`
  - **Description**: The `*mut ContextInformation` return values from the original functions are omitted in the verified model. This leaves the low-level context switch mechanism unverified.
  - **Suggested Fix**: Ensure this scope limitation is clearly documented (it is mentioned in the comments, which is good). Long-term, consider modeling the context pointer as an opaque ghost type to verify it is passed correctly.

## Positive Observations
- **Strong Specification Coverage**: All key state transitions (`sleep`, `schedule`, `exit`) are formally specified and verified.
- **Robust Invariants**: The proofs convincingly establish that thread identity and well-formedness are preserved across all transitions.
- **Clear Abstraction Model**: The documentation effectively explains the mapping from kernel types (`Box`, `SystemTime`, `ExitStatus`) to Verus primitives (`ThreadState`, `Option<int>`, `int`).
- **Clean Structure**: The separation into `running.rs`, `running.spec.rs`, and `running.proof.rs` is clean and maintainable.

## Summary
The verification of `running_thread` is high-quality, successfully proving the correctness of thread state transitions and identity preservation. The abstraction level is appropriate for the component's role. The identified issues are primarily API divergences (missing method, signature changes) that reduce the strict equivalence between the verified model and the original code. Fixing the `take_mutex_guard` signature and restoring `join_cond` would significantly improve the fidelity of the verification.
