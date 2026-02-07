# Review: spinlock (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- **Location:** `LockToken` / `unlock()` (spec/exec: `spinlock.spec.rs`, `spinlock.rs`)
  **Description:** `LockToken` only carries `SpinlockView { locked: bool }`, so tokens are not tied to a specific lock instance. Because `unlock()` only requires `token.view == old(self)@`, any token from any locked spinlock can unlock any other locked spinlock, violating ownership and mutual exclusion.
  **Suggested Fix:** Extend the view with a unique identity (e.g., ghost `id` or permission), and require `token.view.id == self@.id` (or equivalent). Maintain and prove invariants linking the identity to each lock instance and the token provenance.

### High
- **Location:** Missing `SpinlockGuard`/`Drop` (exec: `spinlock.rs`)
  **Description:** The verified module does not implement the `SpinlockGuard` type or the `Drop` behavior from the original. The API is replaced by explicit `unlock()` with tokens, so coverage and RAII semantics are not verified.
  **Suggested Fix:** Provide a verified guard type that models `Drop` (possibly via a trusted wrapper), or prove a refinement between the guard-based API and the token-based API.
- **Location:** `lock()` signature and precondition (exec: `spinlock.rs`)
  **Description:** The verified `lock(&mut self)` requires the lock to be unlocked, while the original `lock(&self)` spins until it becomes unlocked. This changes the semantics and removes the concurrent-access behavior central to a spinlock.
  **Suggested Fix:** Model shared access (e.g., via atomic state and permissions) and remove the unlocked precondition, or explicitly prove a refinement that the sequential model captures the concurrent behavior under fairness assumptions.

### Medium
- **Location:** `lock()` uses `external_body` (exec: `spinlock.rs`)
  **Description:** The core lock acquisition is trusted, so the postconditions are assumed without proof. This is a major trust boundary in a kernel sync primitive.
  **Suggested Fix:** Implement `lock()` as a verified loop over `try_lock()` with an explicit progress assumption, or move the trusted portion into a clearly documented HAL layer and keep the protocol proof in-core.
- **Location:** Invariants (`wf()` and token/state linkage) (spec/proof: `spinlock.spec.rs`, `spinlock.proof.rs`)
  **Description:** The invariant is trivial and does not relate lock state to token ownership or exclusivity, so key safety properties (e.g., “locked implies a unique owner”) are not captured.
  **Suggested Fix:** Add ghost state tracking ownership (e.g., tracked `Option<LockToken>` or permission tokens) and prove uniqueness/ownership invariants preserved by `lock`/`unlock`.

### Low
- **Location:** Liveness/progress (spec/proof: `spinlock.rs`, `spinlock.proof.rs`)
  **Description:** No liveness or fairness property is specified or proven; termination of spinning is unmodeled.
  **Suggested Fix:** State explicit liveness assumptions or add a progress lemma under fairness conditions if such properties are required.

## Positive Observations
- Clear separation between spec, proof, and exec files, with good documentation of modeling choices.
- The protocol state machine (locked/unlocked) is consistently specified and verified in the sequential model.
- The token-based approach is a reasonable technique to model RAII obligations when `Drop` cannot be verified directly.

## Summary
The verification captures a sequential state machine but misses key equivalence and ownership properties of the original spinlock, especially identity/guard semantics and concurrent access. Addressing the token identity linkage and guard/drop coverage would significantly improve soundness. Consider tightening invariants and reducing trust in `external_body` for core locking.
