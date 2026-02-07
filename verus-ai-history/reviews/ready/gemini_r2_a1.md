# Review: ready (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### High
- **Unverified Mutable Access**: `thread_state_mut` is marked `#[verifier::external]`. This allows callers to mutate `ThreadState` without verification checks, potentially violating invariants (e.g., `wf`, `spec_id`).
  - **Location**: `ready.rs`, `thread_state_mut`
  - **Fix**: This is likely unavoidable due to current Verus limitations regarding `&mut T` return types. Ensure all call sites are strictly audited as noted in the documentation. Long-term, prefer extending the "verified forwarding methods" pattern (like `set_interrupt_reason`) to cover all necessary mutations.

### Medium
- **Missing Public API**: `join_cond()` is present in the original source but missing in the verified model.
  - **Location**: `ready.rs`
  - **Fix**: Add a verified `join_cond()` method. If `Condvar` is opaque, it can return an abstract model or be marked `external_body` / `trusted` to maintain API parity.

### Low
- **Signature Mismatch in `run()`**: The verified `run()` returns a `RunResult` struct and omits the `*mut ContextInformation` raw pointer found in the original return tuple.
  - **Location**: `ready.rs`, `run`
  - **Fix**: This is a valid abstraction for verification (hiding HAL details), but ensure that the integration layer (scheduler) can handle the missing pointer, or that the verified code is only used as a model and not a drop-in replacement yet.

## Positive Observations
- **Strong Specification**: The `ReadyThread` state transitions (especially `run` and `terminate`) are well-specified, capturing the movement of interrupt reasons and exit statuses.
- **Verification-Friendly API Extensions**: The verified code introduces safe forwarding methods (`set_interrupt_reason`, `store_mutex_guard`) to avoid using the unsafe `thread_state_mut` accessor. This is an excellent pattern for incremental verification.
- **Clean Separation**: The split between `ready.rs` (exec), `ready.spec.rs` (views/specs), and `ready.proof.rs` (lemmas) is well-organized and easy to navigate.
- **Explicit Trust Boundaries**: The documentation clearly enumerates what is trusted (clock, HAL types), which aids in auditing.

## Summary
The `ready` module is well-verified with a strong focus on state transition correctness. The critical paths for thread scheduling (`run`) and termination (`terminate`) are proven to preserve well-formedness and identity. The primary risks are the `thread_state_mut` escape hatch and the omission of `join_cond`, but the former is well-documented and the latter is a minor API gap. The use of verified forwarding methods to reduce reliance on the external mutable accessor is a commendable design choice.
