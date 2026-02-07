# Review: spinlock (gpt-5.2-codex)

## Grade: C

## Issues Found

### High
- **Location:** Identity uniqueness for `LockToken` / `Spinlock::new` (exec/spec: `spinlock.rs` lines 78-113; `spinlock.spec.rs` lines 13-48)
  **Description:** The fix adds `id` to `SpinlockView` and binds `LockToken.view` to `self@` (good), but uniqueness of `id` is only documented (“Callers must provide a unique `id`”) and not enforced or proven. Nothing prevents constructing two locks with the same `id`, which would still allow a token from one instance to unlock the other via `token.view == old(self)@` because the view only contains `locked` and `id`. This is the same safety hole as before unless a global uniqueness invariant is assumed.
  **Evidence:** `Spinlock::new(Ghost(id))` has no `requires` about uniqueness (spinlock.rs lines 98-110). `wf()` is trivial (spinlock.spec.rs lines 65-75), so no invariant prevents reuse.
  **Suggested Fix:** Add and propagate a global uniqueness assumption/invariant (e.g., a ghost allocator for IDs) or encode ownership via a tracked permission rather than a free `id`.

- **Location:** `lock()` semantics and precondition (exec: `spinlock.rs` lines 152-179)
  **Description:** `lock(&mut self)` still requires `old(self).spec_is_unlocked()` and is marked `external_body`, so the verified model does not capture spinning or concurrent access. This is a functional divergence from the original `lock(&self)` API and weakens the verification to a sequential preconditioned model.
  **Suggested Fix:** Prove a refinement from the sequential model to the concurrent API, or model `lock(&self)` with an explicit progress/fairness assumption and verified loop.

### Medium
- **Location:** Guard/Drop semantics (exec: `spinlock.rs` lines 167-193)
  **Description:** The verified API still does not provide a `SpinlockGuard` type or verified `Drop`. The token-based obligation is a reasonable modeling technique, but the original RAII semantics are not shown to refine to the token API, and clients could still misuse tokens if they are allowed to ignore the guard pattern.
  **Suggested Fix:** Provide a verified guard wrapper or a refinement lemma connecting the guard API to the token model.

- **Location:** Ownership/uniqueness invariants (spec: `spinlock.spec.rs` lines 65-75; proof: `spinlock.proof.rs`)
  **Description:** `wf()` remains trivial and there is no tracked state inside the lock to enforce “locked implies exactly one outstanding token.” The token is linear, but nothing ties it to the lock’s internal state beyond view equality, and no global uniqueness invariant is established.
  **Suggested Fix:** Add a tracked ownership field or invariant proving uniqueness of outstanding tokens per lock and integrate it into `wf()`.

### Low
- **Location:** Liveness/progress (exec/spec)
  **Description:** No liveness or fairness assumptions are specified; termination of spinning is still unmodeled.
  **Suggested Fix:** State explicit progress assumptions or prove a liveness lemma under fairness.

## Positive Observations
- The token now carries a view with an `id`, and `unlock()` requires `token.view == old(self)@`, which addresses the *direct* cross-lock unlock problem if IDs are unique.
- The documentation clearly explains the sequential modeling choice and the trust boundary for `lock()`.

## Summary
The prover partially fixed the identity issue by adding a ghost `id`, but uniqueness is only documented, not enforced or proven. Core divergences remain: `lock()` is still `external_body` with an unlocked precondition and the RAII guard semantics are not verified. Verification is improved but still incomplete for the original spinlock API.
