# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- **Location:** `Mutex::lock` (exec: verus/split/kernel/pm/sync/mutex.rs)
  **Description:** The verified `lock()` still requires `spec_is_unlocked()` and
  explicitly models only the uncontended, instant-success case; contended blocking
  behavior of the real mutex is still outside the model. This is unchanged from the
  prior review—the update documents the limitation but does not fix the semantic gap.
  **Evidence:** `lock()` preconditions require `spec_is_unlocked()` and `!token_issued()`
  (mutex.rs lines 242-246); the doc states contended locking is out of scope (lines 232-236).
  **Suggested Fix:** Model blocking semantics or provide a separate
  `lock_uncontended()` API with a verified refinement to the real `lock()`.
- **Location:** Sequential model / atomicity (exec/spec)
  **Description:** The model still uses `&mut self` with a plain `bool` and explicitly
  excludes concurrency and atomicity, so mutual exclusion under concurrent access is
  not proven. This remains a critical coverage gap, and the new documentation does not
  change the verification claim.
  **Evidence:** The verification scope explicitly excludes concurrency/atomicity
  (mutex.rs lines 41-45).
  **Suggested Fix:** Introduce a verified atomic/CAS or rely-guarantee model, or
  restrict and prove a sequential-only API with a justified refinement layer.

### High
- **Location:** `MutexToken` (spec: verus/split/kernel/pm/sync/mutex.spec.rs)
  **Description:** `MutexToken` still exposes `pub ghost view`, so external code can
  fabricate tokens and satisfy `unlock()` preconditions without holding the lock.
  This is now documented as a trust assumption (T3) but remains a soundness hole.
  **Evidence:** `pub ghost view` field in `MutexToken` (mutex.spec.rs lines 43-46) and
  T3 note in mutex.rs lines 110-113.
  **Suggested Fix:** Make token construction private/opaque (sealed tracked type or
  module-private constructor).
- **Location:** Mutex identity (`id`) uniqueness (exec/spec)
  **Description:** Uniqueness of the ghost `id` is still assumed, not enforced. If two
  mutexes share an `id`, tokens can unlock the wrong instance. This is still a soundness
  assumption rather than a verified invariant.
  **Evidence:** T1 assumption (mutex.rs lines 103-106) and public `id: Ghost<nat>` field
  (mutex.rs lines 148-150).
  **Suggested Fix:** Provide a verified allocator or a non-duplicable ownership resource.

### Medium
- **Location:** API coverage gaps (exec: verus/split/kernel/pm/sync/mutex.rs)
  **Description:** `reference_count()`, `MutexGuard::drop()`/`unlock_unchecked()`, and
  timeout/SleepError/notify error behavior remain unmodeled; the update only documents
  these omissions. The verified surface still does not cover several safety-relevant
  behaviors of the original API.
  **Evidence:** API mapping and divergence notes (mutex.rs lines 56-88).
  **Suggested Fix:** Add verified wrappers/specs for these behaviors or clearly separate
  a verified subset API from the full runtime interface.

## Positive Observations
- Documentation is clearer about the sequential scope, trust assumptions, and API
  mapping.
- Added protocol lemmas help regression-test the spec and clarify intended invariants.

## Summary
The update mostly documents existing limitations rather than fixing them. The
verification remains a sequential state-machine model with significant trust
assumptions (token fabrication and id uniqueness) and does not establish the core
concurrent correctness or blocking semantics of the real kernel mutex.
