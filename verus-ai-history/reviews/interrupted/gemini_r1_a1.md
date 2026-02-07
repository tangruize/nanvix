# Review: interrupted (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **External Mutability**: `thread_state_mut` is marked `#[verifier::external]` in `interrupted.rs`.
  - **Description**: Verus does not currently support returning `&mut T`, so this function is excluded from verification. This prevents verified code from mutating the thread state via this accessor and creates a soundness gap where unverified callers could violate the `wf()` invariant of the `InterruptedThread`.
  - **Suggested Fix**: If verified clients need to mutate the state, implement specific verified setter methods on `InterruptedThread` (wrapping `state` mutations) instead of exposing the raw mutable reference.

### Low
- **Missing Functionality**: `join_cond` is omitted from the verified implementation.
  - **Description**: The original source includes a `join_cond` method that delegates to `ThreadState`. It is omitted here because `Condvar` is treated as an opaque type.
  - **Suggested Fix**: None required if synchronization is out of scope, but it should be noted that this module cannot verify properties related to thread joining.
- **Boundary Model Limitation**: `ReadyThread` boundary model omits fields.
  - **Description**: The local definition of `ReadyThread` omits the `admission_time` field present in the actual implementation.
  - **Suggested Fix**: Ensure the full `ReadyThread` module verification (in `ready.rs`) correctly handles the initialization of `admission_time` when transitioning from `InterruptedThread`, as this model assumes it's irrelevant for safety.

## Positive Observations
- **Strong Specification**: The specs correctly model the `InterruptReason` variants and enforce valid transitions.
- **Explicit Safety Proofs**: The module explicitly proves key safety properties, such as "resume correctly stamps the interrupt reason" and "resume preserves thread identity."
- **Clean Split**: The separation into `exec`, `spec`, and `proof` files is clean and follows best practices.
- **Documentation**: Trust boundaries and modeling assumptions (e.g., why `Condvar` is omitted) are clearly documented in the code.

## Summary
The verification of `interrupted` is solid and captures the essential logic of the state transition. The specifications are sound (within the documented trust boundaries), and the proofs cover the critical invariants of identity preservation and reason propagation. The primary limitations—missing mutable accessors and synchronization primitives—are due to current tool constraints and are well-justified in the comments.
