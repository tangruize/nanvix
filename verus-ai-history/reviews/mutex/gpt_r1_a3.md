# Review: mutex (gpt-5.2-codex)

## Grade: C

## Previous Issues Check

### High: `Mutex::lock` contended behavior and SleepError handling
- **Status:** Not fixed.
- **Evidence:** `Mutex::lock` still requires `spec_is_unlocked()` and calls `try_lock()` once; no wait/loop/timeout modeled (`verus/split/kernel/pm/sync/mutex.rs`, lines ~221-249). The documentation still marks blocking/timeout as out of scope.
- **Impact:** Core blocking semantics and liveness remain unverified.

### High: Concurrency/atomicity model mismatch
- **Status:** Not fixed.
- **Evidence:** Model remains sequential (`&mut self`, plain `bool`); no atomic/linearization/refinement proof added (mutex.rs lines ~24-75).
- **Impact:** Mutual exclusion is only sequential; real concurrent correctness is unverified.

### Medium: Missing API coverage (`reference_count`, `unlock_unchecked`, `MutexGuard::drop`, `Debug`)
- **Status:** Not fixed.
- **Evidence:** API mapping still lists these as not modeled (mutex.rs lines ~58-66). No spec/proof stubs added in `mutex.spec.rs`/`mutex.proof.rs`.
- **Impact:** Behavior for these functions remains out of scope.

### Medium: Mutex identity uniqueness not enforced
- **Status:** Not fixed.
- **Evidence:** `id` uniqueness is still a trust assumption (mutex.rs lines ~102-105, ~152-170). No global allocator/registry introduced in spec or proof.
- **Impact:** Token isolation can be violated if IDs collide.

### Low: Unlock ignores `Condvar::notify_first` behavior
- **Status:** Not fixed.
- **Evidence:** `unlock` still only flips lock/token with no modeled notify or error path (mutex.rs lines ~252-283). Documentation still notes this is out of scope.
- **Impact:** Wakeup behavior remains unverified.

## New Issues Found

- None. The updates appear to add documentation and proof lemmas (e.g., contention protocol), but do not change the modeled semantics.

## Verification Completeness and Soundness

- **Still incomplete for the runtime mutex.** The verification is a sequential protocol model with explicit trust assumptions and excludes concurrency, blocking semantics, and missing APIs. The new lemmas do not address the previously identified gaps.

## Summary

The prior issues were not fixed; they were largely reaffirmed by documentation and additional lemmas. The verification remains a sequential state-machine proof and does not establish correctness for the actual concurrent, blocking mutex implementation.
