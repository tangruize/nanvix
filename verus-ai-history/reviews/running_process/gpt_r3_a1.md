# Review: running_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Invariant weakness (exec/spec)**: `RunningProcess::wf()` only checks counter/sequence length equality and does not enforce thread ID uniqueness or disjointness across running/ready/interrupted/sleeping/zombie lists (`running.spec.rs::wf`). This allows states where the same thread appears in multiple lists, which breaks essential safety properties (e.g., a thread can be scheduled twice or “joined” while still running). **Suggested Fix:** Strengthen `wf()` to include disjointness (use `wf_strict()`), or require `wf_strict()` as a precondition for all public methods and prove preservation.
- **Trusted external resume (exec)**: `interrupted_resume()` is `#[verifier::external_body]` but carries strong guarantees (ready list length 1, head/tail preservation) used in `sleep()`, `exit()`, and `exit_thread()` (`running.rs::interrupted_resume`). If the interrupted module is not yet verified, the core transition proofs are effectively assumed. **Suggested Fix:** Verify `InterruptedProcess::resume()` and replace `external_body` with a verified wrapper/lemma, or weaken the assumptions to only what is proven elsewhere.

### Medium
- **Oracle parameters are a soundness gap (exec)**: `wakeup(found)` and `try_join_thread(tag)` rely on caller-provided oracles matching ghost membership (`running.rs::wakeup`, `running.rs::try_join_thread`). If any caller is `external_body`/assumed, the oracle constraints can be violated, invalidating proofs. **Suggested Fix:** Provide verified wrappers that compute `found/tag` from exec state (or prove them in callers), or restrict these functions to trusted, verified call sites.
- **Join semantics under-specified (spec/exec)**: `try_join_thread` only returns a tag and does not model the returned `ZombieThread`, `Condvar`, or error codes (`running.spec.rs::spec_try_join_thread`, `running.rs::try_join_thread`). This is too weak to capture correctness of join results beyond list updates. **Suggested Fix:** Extend the model to include ghost representations of `ZombieThread`/`Condvar` and error codes, and prove correspondence.
- **find_thread/find_thread_mut are spec-only (exec/spec)**: The exec implementations return `Ghost(spec_find_thread)` and do not verify the actual search order or reference returned in the original code (`running.rs::find_thread`, `running.rs::find_thread_mut`). **Suggested Fix:** Add a verified wrapper around a ghosted search over exec structures, or defer with a tracked proof once reference types are expressible.

### Low
- **Timing/sync elisions limit properties (spec/exec)**: `SystemTime` (alarm), `ContextInformation*`, and `Condvar` are elided, so timing/liveness and synchronization properties are not proven. **Suggested Fix:** Document explicitly which liveness/sync properties are out-of-scope, or enrich the model if those properties are required.

## Positive Observations
- All original functions have verified counterparts and are modeled with explicit postconditions.
- Key state-machine transitions (schedule/sleep/exit/exit_thread/wakeup) preserve PID and thread-count invariants and model list contents precisely.
- Proofs include conservation and content lemmas, and the split between exec/spec/proof is clean and well-documented.

## Summary
The verification is strong on state-machine safety and content preservation but leaves essential invariants (thread uniqueness) and key dependencies (`InterruptedProcess::resume`, oracle-driven searches) as trusted assumptions. Tightening invariants and eliminating external-body/oracle reliance would materially improve soundness. Liveness/timing/sync aspects remain out of scope and should be documented or modeled if required.
