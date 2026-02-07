# Review: mutex (gpt-5.2-codex)

## Grade: C

## Previous Issues Check

### High: `Mutex::lock` contended behavior and SleepError handling
- **Status:** Not fixed (only documented as out of scope).
- **Evidence:** `Mutex::lock` still requires `spec_is_unlocked()` and calls `try_lock()` once, with no wait/loop or timeout/error modeling (`verus/split/kernel/pm/sync/mutex.rs`, lines ~221-249). The new documentation explicitly states blocking/timeout are out of scope (lines ~41-79), but the behavior is unchanged.
- **Impact:** Verification still excludes core blocking semantics and liveness behavior of the real mutex.

### High: Concurrency/atomicity model mismatch
- **Status:** Not fixed (only documented as out of scope).
- **Evidence:** The model still uses `&mut self` and a plain `bool` lock, with no linearization or atomic CAS refinement. Documentation now explicitly disclaims concurrency/atomicity (mutex.rs lines ~24-74), but no concurrent model is added.
- **Impact:** Mutual exclusion is only proven in a sequential state machine; OS-kernel correctness under concurrency is not covered.

### Medium: Missing API coverage (`reference_count`, `unlock_unchecked`, `MutexGuard::drop`, `Debug`)
- **Status:** Not fixed (only documented as not modeled).
- **Evidence:** API mapping explicitly says these are not modeled (mutex.rs lines ~58-66). No corresponding spec stubs or proofs were added in `mutex.spec.rs` or `mutex.proof.rs`.
- **Impact:** The verification still omits several functions and behavioral effects (including the notify path in `unlock_unchecked` / `Drop`).

### Medium: Mutex identity uniqueness not enforced
- **Status:** Not fixed (only documented as trust assumption).
- **Evidence:** `Mutex::new` still accepts a caller-provided ghost `id` with no global uniqueness invariant (`mutex.rs` lines ~152-170; spec notes T1 in lines ~102-105). The spec still lacks any global registry/allocator.
- **Impact:** Token isolation can be violated if IDs collide; this is an explicit trust assumption but remains unverifiable.

### Low: Unlock ignores `Condvar::notify_first` behavior
- **Status:** Not fixed (documented as out of scope).
- **Evidence:** `unlock` only flips the lock state and token, with no modeled notify or error path (`mutex.rs` lines ~252-283). Documentation acknowledges the missing notify/error behavior (lines ~84-87).
- **Impact:** Behavioral mismatch remains for wakeup semantics; acceptable only if condvar is verified and linked with an explicit refinement.

## New Issues Found

- None introduced by the changes; the modifications are primarily documentation and proof lemmas.

## Verification Completeness and Soundness

- **Not complete for the runtime mutex.** The verification remains a sequential protocol model with explicit trust boundaries. It does not establish correctness of the real concurrent, atomic, blocking mutex. The new documentation makes these limits clearer but does not resolve them.

## Summary

The prover did not fix the previously identified issues; instead, they documented them as out-of-scope or trust assumptions. This improves transparency but does not increase the verification strength. The core limitations (blocking semantics, concurrency/atomicity, missing APIs, global ID uniqueness, and condvar wakeups) remain and prevent a sound, complete verification of the actual kernel mutex behavior.
