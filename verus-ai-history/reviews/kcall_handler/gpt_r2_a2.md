# Review: kcall_handler (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage/Equivalence** (exec: `kcall_handler_loop`, `poll_messages_gated`): Still no verified wrapper with the original `kcall_handler` signature or proof tying `stdio_enabled` to the compile-time feature. The model remains an indirect shadow without a refinement link to the real handler. **Suggested Fix:** Add a verified wrapper matching the original signature and a refinement lemma connecting `stdio_enabled` to `cfg(feature = "stdio")`.
- **Liveness/Equivalence** (exec/spec/proof: `kcall_handler_loop`, `spec_initd_terminates_within`, `lemma_conditional_liveness_assumption`): The new liveness predicate and lemma are tautological and do not connect to actual loop iterations; the fuel-bounded loop can still return `terminated = false` even if INITD eventually exits. This is not a real fix. **Suggested Fix:** State a conditional termination lemma over the actual loop that assumes a terminating harvest outcome within N iterations and proves `terminated == true` for `fuel >= N`.
- **Soundness** (exec: `event_init`): Still `external_body` with no failure modeling; the panic path on init failure is unverified. **Suggested Fix:** Model `event_init` as returning a `Result` (or a ghost success flag) and reflect the panic/abort behavior in the spec, or add an explicit assumption that initialization cannot fail.
- **Specifications** (exec: `poll_messages_raw`): Return value remains unconstrained, so “yield iff no work” is proven against an arbitrary boolean rather than actual message receipt. **Suggested Fix:** Tie the return value to a ghost message-queue state or add minimal postconditions (e.g., `result ==> message_was_processed`).

### Low
- **Specifications** (exec: `harvest_zombies`): `exit_status` is still unconstrained even on INITD termination, so the model cannot prove the handler returns the real init daemon status. **Suggested Fix:** Constrain `exit_status` when `found && is_initd` or thread a ghost state reflecting the terminated process’ status.

## Positive Observations
- Added explicit liveness assumption predicate in the spec and documented the conditional nature of termination.
- Existing dispatch classification and loop-invariant proofs remain intact after updates.

## Summary
The update mostly adds documentation and a tautological liveness lemma but does not fix the substantive equivalence, soundness, or specification gaps from the prior review. The verification still models a shadow loop with a fuel bound and unconstrained external bodies, so key behavioral properties of the real handler remain unproven.
