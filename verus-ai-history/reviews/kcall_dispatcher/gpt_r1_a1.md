# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- **Location:** do_kcall (exec: verus/split/kernel/kcall/dispatcher.rs)
  - **Description:** The main dispatcher is marked `#[verifier::external_body]` and left `unimplemented!()`, so the core routing logic, subsystem calls, and error handling are not verified. The proofs only cover classification and helper properties, which means equivalence to the original `do_kcall` is assumed rather than proven.
  - **Suggested Fix:** Implement `do_kcall` in the exec model, keeping ProcessManager/ScoreBoard operations as small external bodies, and prove that the match structure and error handling correspond to the original code.

### High
- **Location:** do_kcall postconditions (exec: verus/split/kernel/kcall/dispatcher.rs)
  - **Description:** The spec claims `LocalImmediate` calls (GetPid/GetTid) always succeed, but the original function can return an error if `ProcessManager::get_pid/get_tid` fail before the match. This makes the spec too strong relative to the source.
  - **Suggested Fix:** Add a precondition that PID/TID retrieval cannot fail, or weaken the postcondition to allow an error result in those failure cases.
- **Location:** do_kcall contract vs. Killed path (exec/spec)
  - **Description:** The original code may diverge on `SleepError::Interrupted(Killed)` via `ProcessManager::exit()` and `panic!`, but the `do_kcall` contract always returns a `DispatchResult`. This is a semantic mismatch and omits a key control-flow behavior.
  - **Suggested Fix:** Model divergence explicitly (e.g., split a non-returning path, or add a precondition excluding Killed from all sleep-error returns) and align the do_kcall spec with that model.

### Medium
- **Location:** spec_dispatch_result_constrained / do_kcall ensures (spec+exec)
  - **Description:** For LocalSleepable/Fallible/Direct/Remote categories, the spec imposes no behavioral constraints beyond well-formedness. This is too weak to establish correct error translation, correct usage of `handle_sleep_error`, or correct scoreboard dispatch behavior.
  - **Suggested Fix:** Strengthen postconditions to relate results to the expected subsystem outcomes (e.g., ok/error mapping for fallible calls, `handle_sleep_error` mapping for sleepable calls, and scoreboard dispatch behavior for remote calls).
- **Location:** Spec constants for KcallNumber values (spec: verus/split/kernel/kcall/dispatcher.spec.rs)
  - **Description:** The spec hardcodes numeric constants for `KcallNumber` without a link to the real enum definitions, so changes in the source enum could silently invalidate the verification.
  - **Suggested Fix:** Import or assert consistency with the actual `KcallNumber` representation (e.g., via a checked mapping or generated constants) to prevent drift.

### Low
- None.

## Positive Observations
- Classification logic mirrors the original match table and is proven total and partitioned (local vs. remote).
- `handle_sleep_error` is specified and proven for the non-divergent cases, including error-code preservation and timed-out mapping.
- Exec/spec/proof separation is clean and consistent, with well-formedness and encoding invariants captured explicitly.

## Summary
The verification captures useful classification and error-mapping properties but leaves the core dispatcher behavior unverified via an `external_body`, and the `do_kcall` spec is both too strong (GetPid/GetTid always succeed) and too weak (no constraints for most categories). Strengthening the `do_kcall` model and aligning divergence and preconditions with the original code would materially improve soundness and equivalence.
