# Review: running_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `RunningProcess::wf` (spec, `running.spec.rs`)  
  **Description:** The well-formedness predicate still only equates counters with sequence lengths and does **not** enforce disjointness/uniqueness of thread IDs across running/ready/interrupted/sleeping/zombie lists. The optional `wf_strict()` exists but is not required by any public function, so the core proofs still allow aliasing a thread across multiple lists. This means key safety properties (e.g., no double-scheduling/joining) remain unproven.  
  **Suggested Fix:** Strengthen `wf()` to include `wf_strict()` disjointness, or require `wf_strict()` as a precondition for all public methods and prove preservation.
- **Location:** `interrupted_resume` (exec, `running.rs`)  
  **Description:** `interrupted_resume()` remains `#[verifier::external_body]` with strong guarantees (exact head/tail preservation). This is a core transition used by `sleep()`, `exit()`, and `exit_thread()`. Without a verified implementation, these properties are assumed rather than proven.  
  **Suggested Fix:** Verify `InterruptedProcess::resume()` in the interrupted module and replace the external body with a proven lemma/wrapper, or weaken the postconditions to match proven properties elsewhere.

### Medium
- **Location:** `wakeup` / `try_join_thread` (exec, `running.rs`)  
  **Description:** Oracle parameters (`found`, `tag`) remain, so correctness still hinges on verified callers supplying correct values. If any caller is `external_body` or assumed, the model can be violated. This is unchanged from the prior review.  
  **Suggested Fix:** Provide verified wrappers that compute the oracle from exec state, or prove oracle constraints in all call sites.
- **Location:** `spec_try_join_thread` / `try_join_thread` (spec/exec, `running.spec.rs`, `running.rs`)  
  **Description:** The update only introduces named tag constants, but still does not model the returned `ZombieThread`, `Condvar`, or error codes. Thus the spec remains too weak to prove correctness of join results beyond list updates.  
  **Suggested Fix:** Add ghost models for join outputs (condvar/zombie/error) and prove correspondence with the original behavior.
- **Location:** `find_thread` / `find_thread_mut` (exec/spec, `running.rs`, `running.spec.rs`)  
  **Description:** These remain spec-only; exec functions return `Ghost(spec_find_thread)` without proving the actual search order or reference selection in the real implementation. The rejection of this issue is not justified by any new proof artifact.  
  **Suggested Fix:** Add a verified search over exec structures or a dedicated lemma when reference-returning functions become expressible.

### Low
- **Location:** timing/sync/HAL elisions (spec/exec, `running.rs`, `running.spec.rs`)  
  **Description:** `SystemTime`, `ContextInformation*`, and `Condvar` remain elided, so timing/liveness and synchronization properties are still unproven. This remains an acknowledged modeling gap.  
  **Suggested Fix:** Explicitly scope these as out-of-scope, or enrich the model if these properties are required.

## Positive Observations
- Coverage of original functions remains complete, and postconditions still capture the main state-machine transitions.
- The join tag constants improve readability and reduce ambiguity in the spec.
- No regressions in schedule/sleep/exit/exit_thread structural invariants were observed.

## Summary
Most of the prior issues remain unfixed: invariants are still too weak, core behavior still depends on an external-body resume, and oracle-driven functions remain a soundness gap. The join-tag refactor is cosmetic and does not strengthen correctness. Overall soundness is still partial.
