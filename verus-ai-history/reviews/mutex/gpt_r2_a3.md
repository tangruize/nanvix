# Review: mutex (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- **Location:** `Mutex::lock` (exec: verus/split/kernel/pm/sync/mutex.rs)
  **Description:** The verified `lock()` still models only the uncontended,
  instant-success case; contended blocking semantics remain out of scope.
  No code changes addressing this were found.
  **Evidence:** `lock()` preconditions require `spec_is_unlocked()` and
  `!token_issued()` (mutex.rs lines 242-246), and the doc reiterates the out-of-scope
  contended case (lines 232-236).
  **Suggested Fix:** Model blocking semantics or introduce a verified
  `lock_uncontended()` plus refinement to the real API.
- **Location:** Sequential model / atomicity (exec/spec)
  **Description:** The model still uses `&mut self` with a plain `bool` and explicitly
  excludes concurrency and atomicity, so mutual exclusion under concurrent access is
  not proven. This remains a core coverage gap.
  **Evidence:** Verification scope explicitly excludes concurrency/atomicity
  (mutex.rs lines 41-45).
  **Suggested Fix:** Add a verified atomic/CAS or rely-guarantee model, or prove a
  refinement from a sequential-only API to the concurrent implementation.

### High
- **Location:** `MutexToken` (spec: verus/split/kernel/pm/sync/mutex.spec.rs)
  **Description:** `MutexToken` still exposes `pub ghost view`, allowing external
  fabrication of tokens that satisfy `unlock()` preconditions. This remains a
  soundness hole; it is documented but not fixed.
  **Evidence:** `pub ghost view` field (mutex.spec.rs lines 43-46) and T3 note in
  mutex.rs lines 110-113.
  **Suggested Fix:** Make token construction private/opaque (sealed tracked type or
  module-private constructor).
- **Location:** Mutex identity (`id`) uniqueness (exec/spec)
  **Description:** Uniqueness of the ghost `id` is still assumed, not enforced.
  If two mutexes share an `id`, tokens can unlock the wrong instance.
  **Evidence:** T1 assumption (mutex.rs lines 103-106) and public `id: Ghost<nat>`
  field (mutex.rs lines 148-150).
  **Suggested Fix:** Provide a verified unique-id allocator or non-duplicable
  ownership resource.

### Medium
- **Location:** API coverage gaps (exec: verus/split/kernel/pm/sync/mutex.rs)
  **Description:** `reference_count()`, `MutexGuard::drop()`/`unlock_unchecked()`,
  and timeout/SleepError/notify error behavior remain unmodeled. The update does not
  add verified counterparts.
  **Evidence:** API mapping and divergence notes (mutex.rs lines 56-88).
  **Suggested Fix:** Add verified wrappers/specs or clearly separate a verified
  subset API from the full runtime interface.

## Positive Observations
- Documentation remains clear about scope and trust assumptions.
- Protocol lemmas still provide regression checks for the state-machine model.

## Summary
No substantive fixes were found in the updated files, so the same critical and
soundness issues remain. The verification is still a sequential model with
unresolved trust assumptions and does not establish concurrent correctness.
