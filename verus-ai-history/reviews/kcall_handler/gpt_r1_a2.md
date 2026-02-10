# Review: kcall_handler (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `kcall_handler_lifecycle_step` (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** The lifecycle invariant is not tied to actual harvest outcomes. The step always appends `HarvestOutcome::NoZombie` regardless of what `run_full_iteration()` observed, so the history is synthetic and the invariant is preserved vacuously. This weakens soundness and equivalence claims about the loop history and termination reasoning.
  **Suggested Fix:** Derive the appended `HarvestOutcome` from the actual harvest result (found/error/is_initd) in the iteration, or return a ghost outcome from `run_iteration()`/`run_full_iteration()` and append that.

### Medium
- **Location:** Loop coverage (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** There is still no verified function that models the full loop until termination and returns the final `ExitStatus`. The new `kcall_handler_init()` + `..._lifecycle_step()` only model one step; coverage of the complete `kcall_handler` lifecycle remains partial.
  **Suggested Fix:** Add a verified loop (or recursive model) that repeatedly calls lifecycle steps until termination and returns the exit status, mirroring the original control flow.
- **Location:** IKC polling feature flag (exec: verus/split/kernel/kcall/handler.rs)
  **Description:** `poll_messages()` remains unconditional and unconstrained even in non-`stdio` builds, allowing message work that cannot occur in the original. This is an over-approximation that affects equivalence (e.g., the model can suppress yields due to phantom messages).
  **Suggested Fix:** Gate message polling by the feature flag or add a spec/assumption that forces `poll_messages()` to return false when `stdio` is disabled.

### Low
- **Location:** Spec/exec error modeling (spec vs exec)
  **Description:** `HarvestOutcome` and `ScoreBoardPollOutcome` exist in the spec, but the exec model uses separate structs/flags and does not map them to the spec enums. As a result, spec lemmas about error outcomes are not connected to exec behavior.
  **Suggested Fix:** Use the spec enums directly in exec modeling or add conversion functions with ensures that link exec results to spec outcomes.

## Positive Observations
- The harvest notification semantics are now aligned with the original (work flag only set on successful notify and never on INITD termination).
- Initialization and post-loop drain are now modeled and invoked in the lifecycle step.
- Dispatch classification proofs and yield-iff-idle reasoning remain clean and well separated from exec code.

## Summary
The fixes addressed the harvested-process flag and added a lifecycle scaffold, but the loop history is still synthetic and not derived from actual iteration outcomes. Coverage is improved but still stops short of a full modeled loop, and IKC feature gating remains unmodeled. Tightening the lifecycle step to use real outcomes and adding a full loop model would materially improve soundness and equivalence.
