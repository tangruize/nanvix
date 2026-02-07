# Review: spinlock (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `Spinlock::lock` (exec: `verus/split/kernel/pm/sync/spinlock.rs`).
  **Description:** The verified `lock` requires `spec_is_unlocked()` and performs a single `try_lock` step with no spinning, so it does not model contention, blocking, or any concurrent behavior. This is a stronger precondition than the original `lock(&self)` which spins and eventually succeeds under concurrent unlock, and it omits the core mutual-exclusion/linearizability reasoning that is essential for a spinlock in a kernel.
  **Suggested Fix:** Model the atomic CAS loop at the spec/proof level (e.g., with an abstract linearization point) and relax the precondition to allow locked states, or add a separate proof obligation capturing progress under fairness assumptions; alternatively, explicitly prove a refinement to a concurrent model.

- **Location:** `SpinlockGuard` / `Drop` (original `src/kernel/src/pm/sync/spinlock.rs`, verified exec/spec/proof).
  **Description:** The verified model replaces RAII with an explicit `unlock()` and a ghost `LockToken`, but there is no verified analog of `SpinlockGuard` or its `Drop` behavior. This fails the coverage criterion for the `Drop` implementation and does not prove that guard drop releases the lock as in the original.
  **Suggested Fix:** Add a verified `SpinlockGuard` wrapper (even ghost-only) and a proof that `Drop` (or an explicit `release`) performs the `store(false, Release)` state transition, or add a refinement lemma showing `LockToken` consumption faithfully models guard drop.

### Medium
- **Location:** `Spinlock::new` / `SpinlockView.id` (exec/spec: `verus/split/kernel/pm/sync/spinlock.rs`, `spinlock.spec.rs`).
  **Description:** Token isolation depends on a caller-supplied ghost `id` being globally unique (T1), but this is not enforced or specified as a precondition. In the original code, identity is enforced by reference identity; the verified model introduces an unchecked trust assumption that can invalidate the token isolation property.
  **Suggested Fix:** Add a global ghost allocator or a module-level uniqueness invariant, and state the uniqueness requirement as a formal precondition for constructors and any refinement proofs.

- **Location:** `Spinlock` struct fields (exec: `verus/split/kernel/pm/sync/spinlock.rs`).
  **Description:** Fields are `pub`, allowing external code to mutate `locked`, `id`, or `token_issued` without going through verified methods, which can break `wf()` and invalidate proofs. The original struct encapsulates state via a private tuple field.
  **Suggested Fix:** Use Verus-friendly encapsulation patterns (e.g., private fields with `open spec fn` accessors, or `#[verifier::opaque]` with trusted getters) to prevent arbitrary external mutation while keeping spec access.

### Low
- **Location:** `Spinlock::try_lock` spec (exec: `verus/split/kernel/pm/sync/spinlock.rs`).
  **Description:** The postcondition `result.0 == !old(self).locked` ties success to the pre-state, which is stronger than the atomic CAS behavior under concurrency (where another thread may unlock between observation and CAS). This is acceptable in the sequential model but not equivalent to concurrent semantics.
  **Suggested Fix:** If modeling concurrency, define success relative to the linearization point or an abstract atomic state, not the pre-state.

## Positive Observations
- The split layout is clean: exec/spec/proof are separated, and specs are documented with clear intent and scope.
- `wf()` cleanly captures the token/lock-state invariant and is preserved by `new`, `lock`, `try_lock`, and `unlock`.
- No `assume`/`external_body` is used in the core module, and verification passes with explicit postconditions.

## Summary
The verification is a solid sequential state-machine proof with good token tracking, but it does not capture the essential concurrent semantics of a kernel spinlock, nor the RAII guard/drop behavior. Strengthening the model to include atomicity/linearizability and guard drop coverage would materially improve equivalence and property coverage.
