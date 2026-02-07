# Review: ready (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Coverage**: `join_cond()` is omitted from the verified interface. While the documentation correctly notes it is elided due to being a synchronization boundary (`Condvar`), this leaves a gap in the public API coverage if downstream code needs to access the condition variable.
- **Soundness**: `thread_state_mut()` is marked `#[verifier::external]`. While necessary due to current Verus limitations regarding `&mut T` return types, this creates an unverified escape hatch. The addition of verified forwarding methods (`set_interrupt_reason`, `store_mutex_guard`) mitigates this significantly, but the risk remains for other state mutations.

## Positive Observations
- **Architecture**: The use of "boundary models" for `RunningThread` and `ZombieThread` is a pragmatic approach to handling cross-module dependencies and circular references, allowing `ReadyThread` to be verified in isolation.
- **Specification**: The `run()` state transition is well-specified, correctly capturing the extraction of the interrupt reason and the clearing of the interrupt flag in the resulting running thread.
- **Documentation**: Excellent inline documentation explaining the verification model, trust boundaries, and elided fields (like `ContextInformation` and `FpuState`).
- **Safety**: Good use of verified forwarding methods (`set_interrupt_reason`, `store_mutex_guard`) to encourage safe usage patterns over the raw mutable accessor.

## Summary
The verification of `ReadyThread` is high quality. It accurately models the critical state transitions (`run`, `terminate`) and preserves key invariants like thread identity. The abstractions used for time and external kernel types are appropriate. The identified issues are minor and well-justified by current tool limitations or architectural boundaries.
