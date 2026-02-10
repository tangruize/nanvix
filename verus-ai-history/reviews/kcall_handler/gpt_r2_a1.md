# Review: kcall_handler (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage/Equivalence** (exec: `kcall_handler_loop`): The original `kcall_handler` is modeled by `kcall_handler_loop` with extra parameters (`fuel`, `stdio_enabled`) and no refinement proof tying it to the real function. There is no verified exec wrapper with the original signature, so coverage/equivalence to the actual handler is indirect. **Suggested Fix:** Add a verified wrapper matching `kcall_handler`’s signature and prove it refines the model, or connect `stdio_enabled` to the compile-time feature and expose a refinement lemma.
- **Liveness/Equivalence** (exec: `kcall_handler_loop`): The fuel-bounded loop can return `terminated = false`, a behavior absent in the original infinite loop, so eventual termination on INITD exit is not proved. **Suggested Fix:** Add a conditional lemma (“if INITD terminates within N iterations, the loop returns its status”) and document an explicit liveness assumption, or model an unbounded loop with a fairness assumption.
- **Soundness** (exec: `event_init`): `event_init()` is `external_body` with no failure modeling, so the panic path on initialization failure is unverified. **Suggested Fix:** Model `event_init` as returning a `Result` (or a ghost flag) and include the panic/abort behavior in the spec, or add a clear assumption that initialization never fails.
- **Specifications** (exec: `poll_messages_raw`): The return value is unconstrained, so the proof of “yield iff no work” is relative to an arbitrary boolean rather than actual message receipt. **Suggested Fix:** Add a postcondition tying the return value to a ghost message queue state, or model minimal constraints (e.g., `result ==> message_was_processed`).

### Low
- **Specifications** (exec: `harvest_zombies`): The `exit_status` is unconstrained even on INITD termination, so the model cannot prove that the handler returns the actual init daemon status. **Suggested Fix:** Constrain `exit_status` when `found && is_initd` or thread a ghost state reflecting the harvested process status.

## Positive Observations
- Dispatch classification is fully enumerated and regression-checked against the source kcall numbers.
- Termination condition (INITD only) and yield/work-flag logic are modeled and proven with explicit invariants.
- Spec/proof separation is clean with clear trust-boundary documentation.

## Summary
The verification captures core control-flow safety properties but relies on a shadow model with several unconstrained external bodies and a fuel-bounded loop, leaving notable equivalence and liveness gaps. Strengthening the refinement link to the real handler and tightening external-body contracts would materially improve confidence.
