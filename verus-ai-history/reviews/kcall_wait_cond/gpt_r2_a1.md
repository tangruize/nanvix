# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `wait_cond.rs` (exec), `wait_cond.spec.rs` (spec)
  **Description:** The model hard-codes `u32` inputs and `USIZE_MAX_X86_32()` but does not require or assert at the model boundary that the target is 32-bit. If the code is ever compiled or verified under a 64-bit `usize`, the model can silently diverge (truncation, different MAX sentinel), breaking equivalence.
  **Suggested Fix:** Add an explicit precondition or compile-time guard (e.g., `requires USIZE_BITS() == 32`) to `wait_cond_model` or model parameters with `usize` and use `usize::MAX` in parsing.

- **Location:** `wait_cond.rs` (exec), `wait_cond.proof.rs` (proof)
  **Description:** Liveness is explicitly out of scope; there is no proof that `cond.wait` or `mutex.lock(None)` eventually returns (either by signal, termination, or timeout). For a wait primitive, eventual return is a key property (at least under fairness or timeout assumptions).
  **Suggested Fix:** Add a liveness lemma (or link to an existing scheduler/condvar liveness proof) and thread it as an assumption into this module, making the reliance explicit in the contract.

- **Location:** `wait_cond.rs` (exec), `wait_cond.spec.rs` (spec)
  **Description:** Resource-release invariants are only asserted on “stored-result” returns. In the actual code, `put_cond` runs unconditionally; if it succeeds and a later step fails, the model does not capture that the condvar reference was still released (nor that the mutex was released by `take_mutex_guard`).
  **Suggested Fix:** Strengthen postconditions to assert `spec_cond_ref_released` whenever `put_cond_model` returns `Ok`, and `spec_mutex_released` whenever `take_mutex_guard_model` returns `Ok`, independent of later continuation errors.

### Low
- None.

## Positive Observations
- The exec model mirrors the original control flow and error precedence, including the critical “stored result” behavior and continuation error override.
- Specs clearly enumerate result categories and include robust lemmas for short-circuiting, error propagation, and unreachable timed-out lock.
- Clean spec/proof/exec split with explicit trust boundaries and documented assumptions.

## Summary
The verification is thorough for functional/error semantics and captures the core pipeline behavior, but it relies on implicit 32-bit assumptions and omits liveness and some resource-release invariants on continuation-error paths. Strengthening those contracts would improve long-term soundness and coverage of key OS safety properties.
