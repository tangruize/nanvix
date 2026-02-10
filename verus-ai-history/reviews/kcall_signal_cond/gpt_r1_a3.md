# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Resource-release predicates remain abstract tokens (spec/exec/proof)**: `spec_cond_ref_released` and `spec_cond_slot_returned` are still uninterpreted and not tied to a concrete PM/condvar state model. The docs now clarify this is intentional, but the kcall proof still does not establish slot/refcount correctness—only token propagation from external bodies.
  - **Location**: `signal_cond.spec.rs` (`spec_cond_ref_released`, `spec_cond_slot_returned`), `signal_cond.rs` module docs, `lemma_cond_ref_released_on_get_cond_success` in `signal_cond.proof.rs`.
  - **Suggested Fix**: Link these tokens to a shared abstract state (or explicitly reference the module where they are defined/proven) so resource-release correctness is provable, not just assumed at the boundary.
- **Notify failure still skips `put_cond` (exec/proof)**: The model continues to short-circuit on notify error, leaving the PM slot unreturned. This is documented but still a potential resource leak unless PM cleanup is proven elsewhere.
  - **Location**: `signal_cond_model` in `signal_cond.rs`, `lemma_notify_error_skips_put_cond` in `signal_cond.proof.rs`.
  - **Suggested Fix**: Either call `put_cond` on notify error (and update spec) or prove a PM invariant that guarantees eventual slot reclamation.

### Low
- **Liveness remains out of scope (spec/proof)**: Wake-up liveness is still not proven here and is only assumed in module docs.
  - **Suggested Fix**: Add liveness specs in scheduler/condvar modules and reference them here, or centralize the assumption in a global spec.

## Positive Observations
- The previous documentation overclaim was corrected; the trust-boundary nature of resource tokens is now clearly stated.
- Broadcast semantics remain explicitly specified and propagated through the pipeline.
- Control-flow equivalence and error propagation are still accurately modeled, with verification passing.

## Summary
The update improves documentation clarity, but the verification still relies on abstract resource-release tokens and retains the notify-error slot leak. These gaps keep the assurance below full functional completeness.
