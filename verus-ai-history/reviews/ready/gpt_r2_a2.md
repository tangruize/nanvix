# Review: ready (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Unconstrained `thread_state_mut()` still unsound** (exec): It remains `#[verifier::external]` with no machine-checked postconditions. Comments and forwarding helpers reduce reliance, but any caller can still violate `wf()` or identity without Verus noticing. Until a verified wrapper or trusted postconditions are enforced, this is still a soundness hole.

### Medium
- **`join_cond()` still omitted** (exec/spec/proof): The model still omits the join condition and only documents it as a trust boundary. This leaves behavioral equivalence around joins unverified; at minimum an opaque token/spec stub would be needed for full coverage.
- **`run()` context pointer still dropped** (exec/spec): The raw `*mut ContextInformation` return remains elided. Documentation clarifies the HAL boundary, but the handoff semantics are still unmodeled and could affect safety reasoning if used elsewhere.

### Low
- **`clock_now()` remains minimally specified** (spec/exec): It only ensures non-negative time; no monotonicity/freshness is modeled, so any scheduling properties depending on "now" remain unverifiable.

## Positive Observations
- Trust boundary documentation is clearer and centralized, making the unverified surface explicit.
- Added verified forwarding methods reduce the need for `thread_state_mut()` in common cases.
- Boundary models for RunningThread/ZombieThread now state cross-module obligations more precisely.

## Summary
Some documentation and helper methods improved clarity, but the main soundness gap (`thread_state_mut`) and two coverage omissions (join_cond, context pointer) remain. Verification is therefore not complete or fully sound.
