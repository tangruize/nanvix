# Review: sleeping_thread (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **Unverified Mutable Access**: `thread_state_mut` is marked `#[verifier::external]`, allowing unverified mutation of the underlying `ThreadState`.
  - **Location**: `impl SleepingThread::thread_state_mut` (exec)
  - **Description**: Verus cannot verify functions returning `&mut T` easily. This function exposes the internal state to arbitrary modification by callers, potentially violating `wf()` invariants (e.g., breaking `spec_id` immutability).
  - **Suggested Fix**: Encapsulate required mutations into specific verified methods on `SleepingThread` (like `set_thread_data_area` was done). If arbitrary mutation is required, this remains a trusted boundary that must be audited manually.

### Low
- **Duplicated Boundary Models**: `ReadyThread` and `InterruptedThread` are redefined locally as boundary models.
  - **Location**: `struct ReadyThread`, `struct InterruptedThread` (exec/spec)
  - **Description**: These structs mimic types from `ready.rs` and `interrupted.rs`. If the real definitions change (e.g., adding a new invariant), these local models might become unsound. The code includes "CROSS-MODULE-CHECK" comments, which is good, but the structural duplication is a maintenance risk.
  - **Suggested Fix**: Eventually move shared boundary models to a common verified module (e.g., `kernel::pm::thread::types`) to ensure a single source of truth.

- **Omitted Function**: `join_cond` is omitted from verification.
  - **Location**: `SleepingThread::join_cond` (original source)
  - **Description**: The function is present in the source but missing in the verified module. The comments explain this is due to `Condvar` being an opaque sync primitive.
  - **Suggested Fix**: Keep as is, but ensure `Condvar` usage is audited separately since it falls outside the verification scope.

- **Duplicated Trusted Function**: `clock_now` is defined as `external_body`.
  - **Location**: `fn clock_now` (exec)
  - **Description**: This function is duplicated from `ready.rs`. It models `SystemTime` as `int`.
  - **Suggested Fix**: Extract to a shared `kernel::util::clock` verified module to avoid duplication.

## Positive Observations
- **Strong Specification**: The specs for `wakeup` and `interrupt` correctly enforce identity preservation, mutex accounting, and drop safety across state transitions.
- **Good Split Architecture**: The separation into `exec`, `spec`, and `proof` files is clean and follows best practices.
- **Verification Success**: The module verifies successfully with no errors.
- **Documentation**: Trust boundaries and cross-module obligations are well-documented in the code.

## Summary
The verification of `sleeping_thread` is high quality. It covers the core state transitions (Sleep -> Ready, Sleep -> Interrupted) with strong guarantees about resource safety and identity preservation. The main weakness is the `external` mutable access to the thread state, which opens a backdoor for invariant violations, though this is a known limitation of the current toolchain. The usage of boundary models is necessary for modular verification but requires careful cross-check management.
