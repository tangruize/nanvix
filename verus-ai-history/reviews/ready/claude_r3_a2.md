# Review: ready (claude-opus-4.6) — Round 2

## Grade: A

## Previous Issue Disposition

### High: Pub struct fields weaken encapsulation

**Verdict: FIXED — Verified.**

The prover added a documentation note at `ready.rs:105-108` explaining that `pub` fields are a Verus modeling necessity and that construction should only occur via `new()`/`from_state()` which establish `wf()`. This directly implements the suggested fix. Confirmed: the note is present and accurate. All exec methods require `wf()` as a precondition, so an ill-formed instance constructed directly would fail precondition checks at any use site. The encapsulation concern is adequately mitigated for Verus verification.

### Medium: EXIT_STATUS_INTERRUPTED hardcoded without mechanical linkage

**Verdict: FIXED — Verified independently.**

The prover added a `CROSS-MODULE-CHECK` comment at `ready.spec.rs:75`. I independently verified the conversion chain: `ErrorCode::Interrupted = EINTR` (`src/libs/error/src/lib.rs:47`), `EINTR: c_int = 4` (`src/libs/sysapi/src/errno.rs:21`), and `From<ErrorCode> for ExitStatus` (`src/libs/sys/src/exit_status.rs:101`) wraps the integer value. The hardcoded `4` is confirmed correct as of the current codebase. The audit trail is now explicit and searchable.

### Medium: Forwarding methods extend API beyond original

**Verdict: FIXED — Verified.**

All three forwarding methods (`set_interrupt_reason` at line 353, `store_mutex_guard` at line 377, `take_mutex_guard` at line 404) are now labeled as **"Verification-only API extension"** with an explicit note that they are not present in the original `ReadyThread`. The semantic divergence is now prominently documented.

### Medium: `clock_now()` lacks monotonicity

**Verdict: ACKNOWLEDGED — Appropriate.**

The reviewer (me) stated "This is acceptable for the current scope." The prover correctly took no action. No change needed.

### Low: `join_cond()` omission

**Verdict: ACKNOWLEDGED — Appropriate.**

No action needed. Already documented. Agreed.

### Low: `run()` context pointer omission

**Verdict: ACKNOWLEDGED — Appropriate.**

No action needed. Already documented. Agreed.

### Low: `thread_state_mut()` escape hatch

**Verdict: ACKNOWLEDGED — Verified.**

The prover claims the AUDIT annotation "already present from prior round." Confirmed: `ready.rs:527` has an AUDIT comment listing known call sites (`process/manager/mod.rs:1154,1169`). The forwarding methods note at lines 510-512 provides a clear mitigation path. Adequate.

### Low: Proof lemma style (structural copy vs exec function)

**Verdict: ACKNOWLEDGED — Appropriate.**

Not a correctness issue. No action needed. Agreed.

## New Issues Found

### Low

- **Location:** `take_mutex_guard` postconditions (`ready.rs:415-422`)
  - **Description:** `store_mutex_guard` explicitly ensures `!self.spec_drop_safe()` (line 394), but `take_mutex_guard` has no postcondition about `spec_drop_safe()` at all. After releasing the last mutex, a thread should become drop-safe (assuming `spec_drop_safe()` is defined as `locked_mutex_count == 0`). This asymmetry means callers cannot determine whether the thread became drop-safe after releasing a mutex, which matters for thread cleanup logic. This is a postcondition completeness gap, not a correctness issue.
  - **Suggested Fix:** Add `self.spec_drop_safe() == (self.spec_locked_mutex_count() == 0)` to `take_mutex_guard`'s postconditions (if the underlying `ThreadState::take_mutex_guard` supports it), or at minimum document the asymmetry.

- **Location:** Forwarding method postconditions (`ready.rs:359-425`)
  - **Description:** The three forwarding methods preserve `spec_id()`, `wf()`, and `spec_admission_time()`, but each omits some cross-cutting field preservation postconditions. For example, `set_interrupt_reason` does not ensure preservation of `spec_kernel_stack()`, `spec_user_stack()`, or `spec_user_tda()`. Similarly, `store_mutex_guard` and `take_mutex_guard` do not ensure preservation of `spec_interrupt_reason()` or `spec_is_interrupted()`. Since `wf()` does not constrain these fields' specific values, callers cannot reason about them after calling these methods. This is a completeness observation, not a correctness issue — all stated postconditions are correct.
  - **Suggested Fix:** If the underlying `ThreadState` method postconditions already guarantee these preservations, consider propagating them through the ReadyThread forwarding methods for proof completeness. Otherwise, document that these fields are preserved in practice but not proven at this layer.

## Positive Observations

- **All previous issues genuinely addressed.** The prover did not deflect — documentation fixes are present and accurate, and acknowledgments are justified. No "not applicable" hand-waving.
- **Independently verified EXIT_STATUS_INTERRUPTED = 4** against the source chain (`EINTR = 4` in `sysapi/errno.rs:21`, `Interrupted = EINTR` in `error/lib.rs:47`). The value is correct.
- **Verification passes cleanly:** 34 verified, 0 errors. No assumes, admits, or external_body on the target functions.
- **Excellent documentation quality persists:** Module-level docs, trust boundary docs, cross-module check annotations, and AUDIT comments form a comprehensive audit trail.
- **Sound trust boundary:** Only 2 `external_body` functions (`clock_now`, `exit_status_interrupted_value`) and 1 `#[verifier::external]` function (`thread_state_mut`), all well-justified.
- **Faithful model:** The verified code accurately captures the original `ReadyThread`'s state transitions (`run()` and `terminate()`), construction paths (`new()` and `from_state()`), and accessor methods. The omissions (raw pointer, Condvar, HAL types) are appropriate abstraction boundaries.

## Summary

All actionable issues from the previous review have been genuinely fixed. The documentation improvements for public fields, the CROSS-MODULE-CHECK annotation for EXIT_STATUS_INTERRUPTED, and the "Verification-only API extension" labels for forwarding methods are exactly what was suggested and are correctly implemented. The acknowledged items were all cases where the previous reviewer explicitly stated no action was needed — the prover correctly did nothing.

Two minor new observations about postcondition completeness in forwarding methods are noted. These are informational — they identify where the verification could be strengthened but do not affect the soundness of the current proof. The verification is complete and sound within its stated scope.
