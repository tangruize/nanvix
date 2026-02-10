# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Resource-release invariants still uninterpreted (spec/proof)**: `spec_cond_ref_released` and `spec_cond_slot_returned` remain uninterpreted with no PM/condvar state model, and the proof still notes the release predicate is a tautology. The updated docs claim these are defined elsewhere, but there are no definitions in the split tree; the verification still doesn’t establish slot/refcount correctness.
  - **Location**: `spec_cond_ref_released`/`spec_cond_slot_returned` in `signal_cond.spec.rs`, `lemma_cond_ref_released_on_get_cond_success` in `signal_cond.proof.rs`, module docs in `signal_cond.rs`.
  - **Suggested Fix**: Introduce a shared abstract state with invariants (or link to an existing state model) and have get/put/drop update it so release/return are provable, not assumed.
- **Notify failure still skips `put_cond` (exec/proof)**: The model continues to short-circuit on notify error, leaving the PM slot unreturned. This was previously flagged and remains a potential resource leak unless PM cleanup is proven elsewhere.
  - **Location**: `signal_cond_model` in `signal_cond.rs`, `lemma_notify_error_skips_put_cond` in `signal_cond.proof.rs`.
  - **Suggested Fix**: Either call `put_cond` on notify error (and update spec) or prove a PM invariant that guarantees eventual slot reclamation.
- **Overclaim in documentation about external verification (exec docs)**: The module doc states that release predicates are “defined and verified” in PM/condvar modules, but they are uninterpreted here and no definitions appear in the split tree. This risks overstating assurance.
  - **Location**: `signal_cond.rs` module docs ("Properties NOT Proven Here").
  - **Suggested Fix**: Remove or qualify the claim, or add explicit references to the modules that define and prove these predicates.

### Low
- **Liveness remains out of scope (spec/proof)**: Wake-up liveness is still not proven here; it is only assumed to be handled by scheduler/condvar modules.
  - **Suggested Fix**: Add liveness specs in those modules and link them here, or clearly state the assumption in a centralized spec.

## Positive Observations
- The broadcast semantics are now explicitly specified and propagated through the pipeline, addressing the prior gap.
- Control-flow equivalence and error propagation remain accurately modeled, and the new lemmas cleanly separate spec vs. proof.
- Verification continues to pass for the updated module.

## Summary
The broadcast semantics issue is fixed, but the verification still relies on uninterpreted resource-release predicates and preserves the notify-error slot leak. Clarifying or proving those invariants would materially improve soundness.
