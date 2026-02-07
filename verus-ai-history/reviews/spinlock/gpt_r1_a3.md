# Review: spinlock (gpt-5.2-codex)

## Grade: C

## Issues Found

### High
- **Location:** Identity uniqueness for `Spinlock::new` / `LockToken` (exec/spec: `spinlock.rs` lines 55-99, 106-126; `spinlock.spec.rs` lines 13-53, 70-91)
  **Description:** The fix still relies on a documented assumption that `id` values are unique (see the new “T1: ID Uniqueness” note). There is still no enforced precondition, allocator, or invariant proving uniqueness, so two locks can share the same `id` and a token from one can satisfy `token.view == old(self)@` for the other. This remains the original cross-instance safety hole.
  **Evidence:** `new(Ghost(id))` has no `requires` for uniqueness and `wf()` only constrains `token_issued` vs `locked` (spinlock.rs lines 106-126; spinlock.spec.rs lines 70-82).
  **Suggested Fix:** Add a ghost allocator or global uniqueness invariant and require it in `new`, or encode ownership in a non-forgeable tracked permission instead of a free `id`.

- **Location:** `lock()` semantics and precondition (exec: `spinlock.rs` lines 171-203)
  **Description:** `lock(&mut self)` still requires the lock to be unlocked and is `external_body`, so the verified model does not cover spinning, concurrency, or the original `lock(&self)` API. This is a material divergence from the production spinlock semantics.
  **Suggested Fix:** Provide a refinement proof from the sequential model to the concurrent API, or verify a loop under explicit progress/fairness assumptions.

### Medium
- **Location:** Guard/Drop semantics (exec: `spinlock.rs` lines 186-215)
  **Description:** A `SpinlockGuard` and verified `Drop` are still absent. The token model is an acceptable abstraction, but there is no refinement lemma tying the RAII API to the token API.
  **Suggested Fix:** Add a verified guard wrapper or prove a refinement lemma showing the guard API is simulated by the token discipline.

- **Location:** Token existence invariant (spec/exec: `spinlock.spec.rs` lines 70-82; `spinlock.rs` lines 96-104, 146-168)
  **Description:** `wf()` only enforces “unlocked ⇒ no token,” but allows “locked ∧ no token.” This means the model permits a locked state with no outstanding token, which breaks the intended “locked implies exactly one token” ownership guarantee. The new `token_issued` ghost field helps, but the invariant is still incomplete.
  **Suggested Fix:** Strengthen `wf()` to `locked ⇔ token_issued` (or at least `locked ⇒ token_issued`), and ensure all operations preserve it.

### Low
- **Location:** Liveness/progress (exec/spec)
  **Description:** No liveness or fairness property is specified; termination of the spin loop remains unmodeled.
  **Suggested Fix:** State explicit progress assumptions or add a liveness lemma under fairness.

## Positive Observations
- The new `token_issued` ghost field plus strengthened preconditions (`!old(self).token_issued`) improves single-token tracking and prevents “unlocked with token outstanding.”
- `unlock()` now consumes token state explicitly and restores `token_issued = false`, which improves protocol clarity.

## Summary
The update improves token bookkeeping but does not fix the fundamental identity-uniqueness hole, and the verified API still diverges from the concurrent `lock(&self)` semantics and RAII guard behavior. Verification remains incomplete and still depends on undocumented global assumptions for soundness.
