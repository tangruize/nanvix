# Review: thread_state (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** Missing exec/spec/proof for `context_mut`, `fpu_state_mut`, `join_cond`, `Debug`, and `Drop` behavior (only documented as out-of-scope).
  **Evidence:** The module documentation explicitly states these functions are omitted from the verification model (`state.rs` lines 40-44, 56-64), and there are no corresponding exec/spec/proof stubs in `state.rs`, `state.spec.rs`, or `state.proof.rs`.
  **Why this matters:** This remains a coverage gap for raw pointer exposure, condvar cloning, debug formatting, and drop-time logging. Documentation alone does not provide verified guarantees.
  **Status vs previous review:** Not fixed; the omission is now documented but still unverified.

- **Location:** `store_mutex_guard` / `take_mutex_guard` (exec `state.rs` lines 261-323; spec/model in `state.spec.rs` lines 15-18).
  **Evidence:** `store_mutex_guard` requires `!old(self).spec_has_mutex(address@)` and always increments count (lines 273-291), while `take_mutex_guard` requires `old(self).spec_has_mutex(address@)` and always decrements count (lines 307-323). This does not model `BTreeMap::insert` overwrite or `remove` returning `None`.
  **Why this matters:** The verification still assumes global invariants (no double-lock, release-only-held) without proving them. This is a soundness gap unless those invariants are established elsewhere.
  **Status vs previous review:** Not fixed; now explicitly documented as a trust assumption but not proven.

### Medium
- **Location:** Drop semantics remain disconnected from exec behavior.
  **Evidence:** A proof lemma links `spec_drop_safe` to empty mutex sets (`state.proof.rs` lines 515-525), but there is still no modeled `Drop::drop()` or logging behavior in the exec/spec surface (`state.rs` lines 40-44, 56-64).
  **Why this matters:** The proof does not show that drop-time checks are enforced or that errors are logged when mutexes remain.
  **Status vs previous review:** Partially improved (lemma added), but still incomplete.

### Low
- None.

## Positive Observations
- The stack identity modeling issue is fixed: stacks are now abstract `Option<int>` tokens with Option::take semantics and identity preservation (`state.rs` lines 88-97, 164-213; `state.spec.rs` lines 64-82).
- The module now clearly documents scope limitations and trust assumptions (`state.rs` lines 46-64), which makes the verification boundaries explicit.
- Drop safety is better connected to mutex set emptiness via a dedicated lemma (`state.proof.rs` lines 515-525).

## Summary
The prover fixed the stack identity modeling and clarified verification scope/assumptions, but two high-impact gaps remain: omitted behavior for context/fpu/join_cond/Drop/Debug and unproven global invariants for mutex guard semantics. The verification is improved but still incomplete and relies on undocumented external guarantees.
