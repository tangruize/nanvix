# Review: running_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `RunningProcess::wf` (spec, `running.spec.rs`)  
  **Description:** The well-formedness predicate still only equates counters with sequence lengths and does **not** enforce disjointness/uniqueness of thread IDs across lists. `wf_strict()` remains optional and is not required by any public method, so aliasing across lists is still permitted. This was not fixed.  
  **Suggested Fix:** Strengthen `wf()` to include disjointness, or require `wf_strict()` as a precondition for all public methods and prove preservation.
- **Location:** `interrupted_resume` (exec, `running.rs`)  
  **Description:** `interrupted_resume()` remains `#[verifier::external_body]` with strong head/tail preservation guarantees used by `sleep()`, `exit()`, and `exit_thread()`. There is still no verified implementation, so the core transition is assumed. This was not fixed.  
  **Suggested Fix:** Verify `InterruptedProcess::resume()` (or replace with a proven wrapper) and discharge the external-body assumption.

### Medium
- **Location:** `wakeup` / `try_join_thread` (exec, `running.rs`)  
  **Description:** Oracle parameters (`found`, `tag`) are still required; correctness hinges on verified callers providing consistent values. If any caller is `external_body`, the proofs remain unsound. No mitigation was added.  
  **Suggested Fix:** Provide verified wrappers that compute the oracle from exec state or prove constraints in all call sites.
- **Location:** `spec_try_join_thread` / `try_join_thread` (spec/exec, `running.spec.rs`, `running.rs`)  
  **Description:** The spec still only models a result tag; it does not model `ZombieThread`, `Condvar`, or error codes. Thus join result correctness beyond list updates remains unproven. No new modeling was added.  
  **Suggested Fix:** Add ghost models for join outputs and prove correspondence with original behavior.
- **Location:** `find_thread` / `find_thread_mut` (exec/spec, `running.rs`, `running.spec.rs`)  
  **Description:** These remain spec-only (exec returns `Ghost(spec_find_thread)`), with no proof of the real search order or reference selection in the original implementation. This was not fixed.  
  **Suggested Fix:** Add a verified search over exec structures or a proof bridge when reference-returning functions are expressible.

### Low
- **Location:** timing/sync/HAL elisions (spec/exec, `running.rs`, `running.spec.rs`)  
  **Description:** `SystemTime`, `ContextInformation*`, and `Condvar` are still elided, leaving timing/liveness and synchronization properties unproven. This remains an acknowledged gap.  
  **Suggested Fix:** Clearly scope these as out-of-scope or extend the model if required.

## Positive Observations
- No regressions detected in schedule/sleep/exit/exit_thread postconditions.
- Coverage of original functions remains complete.

## Summary
The update does not materially address the prior review: invariants remain weak, `interrupted_resume()` is still external, and oracle-driven functions remain trust-based. Verification is still partial and not sound enough for full correctness claims.
