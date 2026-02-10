# Review: kcall_sleep (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Missing timing/liveness properties** (sleep.exec.rs / sleep.spec.rs / sleep.proof.rs):
  The model intentionally omits properties like "TimedOut/Ok implies the alarm time was reached" and
  "ProcessManager::sleep eventually returns." These are key correctness aspects of a sleep syscall
  (timing and progress). While the module documents them as out-of-scope, the kcall verification
  itself does not establish them.
  - **Suggested Fix:** Add postconditions/lemmas (or import them) that tie PM outcomes to alarm-time
    semantics and progress, e.g., a PM spec lemma that `PmTimedOut`/`PmOk` implies the current time
    has reached the alarm and that `process_manager_sleep` is live under scheduler fairness.

- **Cast-safety assumptions are only informal** (sleep.exec.rs: `sleep_model`, `sleep_end_to_end`):
  The verified API takes `(seconds: u64, nanoseconds: u32)` and assumes the original `(usize, usize)`
  casts are safe because Nanvix targets x86-32. This assumption is not enforced in the interface
  (especially for `nanoseconds`), so equivalence relies on target-specific knowledge rather than a
  proved or encoded invariant.
  - **Suggested Fix:** Add a verified wrapper that takes `usize` and proves the casts to `u64/u32` are
    identity on the supported target, or encode the 32-bit ABI assumption as a spec invariant and
    use it in preconditions for both parameters.

### Low
- None.

## Positive Observations
- Full coverage for the original function: the exec model mirrors the 3-arm match and overflow path.
- Specs capture the core control-flow logic, error-code propagation, and classification semantics.
- Proofs cover overflow handling, classification completeness, and well-formedness invariants.
- Split quality is strong: spec types/functions, proofs, and exec logic are cleanly separated.

## Summary
The verification is strong for control-flow classification and overflow handling, with a clear and
well-structured split between exec/spec/proof. The main gaps are the absence of timing/liveness
properties and the reliance on informal cast-safety assumptions; tightening these would raise the
assurance to full end-to-end syscall correctness.
