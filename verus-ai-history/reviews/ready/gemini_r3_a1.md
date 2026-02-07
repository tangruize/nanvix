# Review: ready (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Medium
- **External Mutable Access**: `thread_state_mut` is marked `#[verifier::external]`, bypassing verification for mutable access to the internal `ThreadState`. While documented and likely necessary for HAL interactions (Context/FPU), it creates a soundness gap where invariants could be violated by unverified callers.
    - *Suggested Fix*: Continue migrating usages to verified forwarding methods (like the added `set_interrupt_reason`). For strictly HAL-related mutations, ensure the `audit` requirement is strictly followed.

### Low
- **Omitted API**: The `join_cond` method is missing from the verified implementation. While `Condvar` is opaque to verification, omitting the method prevents verifying callers that might need to access the condition variable.
    - *Suggested Fix*: precise model for `Condvar` isn't needed, but exposing it as an opaque external type would allow complete API coverage.
- **Signature Deviation**: The `run` method in the verified model returns `RunResult`, which omits the `*mut ContextInformation` pointer returned by the original code.
    - *Suggested Fix*: This is acceptable if the pointer is only used in unverified assembly/glue code. If high-level logic uses it, it should be added to `RunResult` (potentially as a `Ghost` or `int` representation).

## Positive Observations
- **Excellent Documentation**: The module contains exceptional documentation regarding the verification model, trust assumptions, and specifically the "Cross-Module Verification Obligations" which clearly state what needs to be checked when sibling modules are verified.
- **Boundary Modeling**: The use of local definition boundary models (`RunningThread`, `ZombieThread`) to decouple verification of circular/complex dependencies is a sophisticated and effective pattern.
- **Verified Helper Methods**: The introduction of verified forwarding methods (`set_interrupt_reason`, `store_mutex_guard`, `take_mutex_guard`) to reduce reliance on the unsafe `thread_state_mut` accessor is a strong design choice.
- **Strong Safety Properties**: The proofs robustly establish preservation of thread identity, well-formedness, and drop-safety (mutex accounting) across state transitions.

## Summary
The `ready` module verification is exemplary. It effectively captures the logic of thread scheduling states (Ready to Running/Zombie) while managing the complexity of kernel dependencies through well-defined boundary models. The specifications are strong, and the separation of proof and code is clean. The reliance on an external mutator for HAL state is the only significant (though likely unavoidable) weakness, and it is handled with appropriate caution and documentation.
