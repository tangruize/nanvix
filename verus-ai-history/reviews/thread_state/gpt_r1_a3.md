# Review: thread_state (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** Missing exec/spec/proof for `context_mut`, `fpu_state_mut`, `join_cond`, and `Debug` behavior.
  **Evidence:** The module still explicitly omits these functions from the verification model (`state.rs` lines 44-48, 66-68), and there are no corresponding exec/spec/proof stubs in the updated files.
  **Why this matters:** Raw pointer exposure, condvar cloning semantics, and debug formatting remain unverified; documenting the omission does not provide soundness guarantees.
  **Status vs previous review:** **Not fixed.**

- **Location:** `store_mutex_guard` / `take_mutex_guard` semantic strengthening (exec `state.rs` lines 265-327).
  **Evidence:** `store_mutex_guard` requires `!old(self).spec_has_mutex(address@)` and always increments count (lines 277-295), while `take_mutex_guard` requires `old(self).spec_has_mutex(address@)` and always decrements count (lines 311-327). This still does not model `BTreeMap::insert` overwrite or `remove` returning `None`.
  **Why this matters:** The verification still assumes the global invariants T1/T2 without proving them elsewhere, so semantic equivalence to the original implementation is not established.
  **Status vs previous review:** **Not fixed.** The new roundtrip lemma strengthens internal consistency but does not discharge the global invariant.

### Medium
- **Location:** Drop behavior is still only partially modeled.
  **Evidence:** `check_drop_safe()` was added to model the `Drop::drop()` check (exec `state.rs` lines 329-346; proof lemma `lemma_check_drop_safe_models_drop` at `state.proof.rs` lines 529-542), but there is still no modeled `Drop::drop()` or verified logging behavior itself.
  **Why this matters:** The model now exposes the check but does not prove that drop actually enforces it or that error logging happens when mutexes remain.
  **Status vs previous review:** **Improved but still incomplete.**

### Low
- None.

## Positive Observations
- `check_drop_safe()` is a meaningful improvement: it ties the runtime counter to `spec_drop_safe()` under `wf()` (`state.rs` lines 329-346; `state.proof.rs` lines 529-542).
- Mutex guard roundtrip lemma was added (`state.proof.rs` lines 549-579), clarifying internal consistency within the trust boundary.
- The prior stack identity modeling remains correct and unchanged.

## Summary
The prover meaningfully improved drop modeling by adding `check_drop_safe()` and a proof lemma, and added a mutex roundtrip lemma, but the two high-impact gaps remain: missing coverage for context/fpu/join_cond/Debug and unproven global mutex invariants. Verification is improved yet still incomplete and relies on external guarantees.
