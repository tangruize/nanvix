# Review: kcall_handler (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage/Equivalence** (exec: `kcall_handler_loop`, `poll_messages_gated`): Still no verified wrapper with the original `kcall_handler` signature or proof tying `stdio_enabled` to the compile-time feature. The model remains an indirect shadow without a refinement link to the real handler. **Suggested Fix:** Add a verified wrapper matching the original signature and a refinement lemma connecting `stdio_enabled` to `cfg(feature = "stdio")`.
- **Liveness/Equivalence** (spec/proof: `spec_initd_terminates_within`, `lemma_loop_termination_completeness`): The new lemmas only restate that the invariant excludes termination from the history; they do not connect actual loop iterations to a fuel-bounded conditional termination property. The fuel-bounded loop can still return `terminated = false` even if INITD terminates later. **Suggested Fix:** Prove a conditional lemma over the actual loop: if a terminating harvest outcome occurs within N iterations, then for `fuel >= N` the loop returns `terminated == true`.
- **Soundness** (exec: `event_init`): Still `external_body` with no failure modeling; the panic path on init failure is unverified. **Suggested Fix:** Model `event_init` as returning a `Result` (or a ghost success flag) and reflect the panic/abort behavior in the spec, or add an explicit assumption that initialization cannot fail.
- **Specifications** (exec: `poll_messages_raw`): Return value remains unconstrained, so “yield iff no work” is proven against an arbitrary boolean rather than actual message receipt. **Suggested Fix:** Tie the return value to a ghost message-queue state or add minimal postconditions (e.g., `result ==> message_was_processed`).

### Low
- **Specifications** (exec: `harvest_zombies`): `exit_status` is still unconstrained even on INITD termination, so the model cannot prove the handler returns the real init daemon status. **Suggested Fix:** Constrain `exit_status` when `found && is_initd` or thread a ghost state reflecting the terminated process’ status.

## Positive Observations
- Added a lemma explicitly relating the invariant to the absence of termination in history.
- Existing dispatch classification and loop-invariant proofs remain intact after updates.

## Summary
The update adds invariant-based lemmas but does not establish the missing refinement or conditional termination properties. Core gaps in equivalence and external-body specifications remain, so verification is still incomplete for key behavioral guarantees.
