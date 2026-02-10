# Review: kcall_handler (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Model vs Implementation Drift**: The verification relies on a model (`run_full_iteration`) that must be manually synchronized with the implementation. While the documentation now clearly acknowledges this and points to regression lemmas (`lemma_spec_constants_match_source`) as a detection mechanism, there is still no automated check (e.g., a build-time script) to guarantee that source constants match the spec constants.
- **Manual Kcall Classification**: The classification logic is duplicated in the spec. This is a maintenance point but does not affect soundness.

## Positive Observations
- **Explicit Limitation Documentation**: The added documentation in `kcall_handler_loop` clearly explains the "Semantic gap" regarding liveness and fuel, which is excellent for auditability.
- **Regression Lemmas**: The use of `lemma_spec_constants_match_source` and `lemma_dispatch_coverage_matches_source` provides a formal contract that the spec expects certain values, making it easier to manually verify against the source.
- **Robust Model**: The model faithfully captures the complex control flow (yield-on-idle, break-on-termination) of the original handler.

## Summary
The verification of `kcall_handler` is solid. The prover has adequately addressed the concerns about model divergence by documenting the limitations and the regression strategy. While a mechanical sync would be ideal, the current approach of "documented model with regression lemmas" is acceptable and standard for this type of kernel verification. The verification successfully proves the key safety properties of the event loop model.
