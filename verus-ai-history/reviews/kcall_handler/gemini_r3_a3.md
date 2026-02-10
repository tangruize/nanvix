# Review: kcall_handler (gemini-3-pro-preview) - Iteration 3

## Grade: B-

## Issues Found

### Critical
- **Shadow Verification (Standard Limitation)**: The verification continues to rely on a shadow model (`verus/split/kernel/kcall/handler.rs`) rather than the actual source. While this is now well-documented and standard for this project, it remains a validity risk. Any change to the source requires manual synchronization with the model.

### High
- (Resolved) **Hardcoded Kcall Numbers**: The spec now correctly uses constants from the dispatcher module (`super::dispatcher::KCALL_*`) instead of raw integers. This ensures the dispatch logic is defined in terms of the system's shared constants.

### Medium
- **Unconstrained External Bodies (Scope)**: `dispatch_to_subsystem` remains loosely specified (`ensures result.is_error ==> result.error_code != 0`). It does not verify that `dispatch(KCALL_DEBUG)` actually invokes the debug subsystem. This is acceptable as a trust boundary definition but leaves the functional correctness of dispatch unverified.
- **Liveness Assumption**: The property that the loop terminates is assumed (`spec_initd_terminates_within`) rather than proven. This is a reasonable scope limitation for a single-module verification but means the "liveness proof" is conditional on an unproven external property.

## Positive Observations
- **Integration with System Constants**: The move to `super::dispatcher::KCALL_*` constants significantly improves the robustness of the spec against constant drift.
- **Sound Loop Logic**: The core logic of the handler loop (work tracking, yield conditions, termination detection) is rigorously verified against the shadow model.
- **Clear Trust Boundaries**: The documentation of T1-T5 trust boundaries is exemplary, clearly stating what is verified vs. what is assumed.

## Summary
The verification of `kcall_handler` is now solid *within the scope of its shadow model*. The critical issue of hardcoded constants has been resolved, making the spec more maintainable and correct by construction. The remaining issues (shadow model, unconstrained dispatch) are documented architectural decisions rather than implementation errors. The grade is upgraded to **B-** to reflect a passing verification with recognized architectural limitations.
