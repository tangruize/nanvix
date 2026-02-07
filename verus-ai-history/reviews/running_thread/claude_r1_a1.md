# Review: running_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

(none)

### Medium

1. **Missing per-address mutex preservation in `exit()` and `ZombieThread` boundary model**
   - **Location:** `exit()` in running.rs (exec), `ZombieThread::from_state` in running.rs (exec)
   - **Description:** The `sleep()` and `schedule()` transitions both include the postcondition `forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a)`, preserving per-address mutex membership through the transition. However, `exit()` only preserves `spec_locked_mutex_count()` and `spec_drop_safe()`, not the per-address `forall` quantifier. The `ZombieThread` boundary model lacks `spec_has_mutex` entirely. While zombie threads are terminal states and per-address info may not be actionable, this is an asymmetry that could matter if a future caller needs to enumerate held mutexes on the zombie (e.g., for diagnostics or cleanup verification).
   - **Suggested Fix:** Add `spec_has_mutex` to `ZombieThread` spec and add `forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a)` to `ZombieThread::from_state` ensures and `exit()` ensures. Alternatively, document the intentional omission.

2. **`take_mutex_guard` return type elision strengthens the contract**
   - **Location:** `take_mutex_guard()` in running.rs (exec), line 401
   - **Description:** The original `take_mutex_guard` returns `Option<MutexGuard>`, allowing callers to handle the `None` case (mutex address not found). The verified version returns unit `()` and requires `old(self).spec_has_mutex(address@)` as a precondition, making the `None` path unreachable by construction. While this is sound (documented as trust assumption T2) and actually *stronger* than the original, it changes the API semantics: callers that pattern-match on `Option` in the original have different control flow than what's verified. If a caller does pass a non-held address at runtime, the original gracefully returns `None` while the verified model has a precondition violation.
   - **Suggested Fix:** This is an acceptable design choice for verification (proving callers hold what they release). Document that runtime callers outside the verification boundary must ensure the precondition holds, or add a postcondition-style model that returns a `bool` indicating whether the mutex was found.

### Low

1. **`join_cond()` omitted without verification alternative**
   - **Location:** Original running.rs line 153, not present in verified code
   - **Description:** `join_cond()` returns a `Condvar` used for thread-join synchronization. It is omitted from verification as a "sync boundary" type. While the omission is documented, join correctness (all waiting threads are woken, no threads left waiting forever — as noted in the original's comment) is an important liveness property that remains unverified.
   - **Suggested Fix:** If thread-join verification is in scope for the broader project, consider adding a ghost protocol model for the condition variable. Otherwise, the current documented omission is acceptable.

2. **`exit()` does not require drop-safety as a precondition**
   - **Location:** `exit()` in running.rs (exec), line 352
   - **Description:** The verified `exit()` allows a thread to transition to zombie while still holding locked mutexes. The original code also allows this (the `Drop` impl on `ThreadState` only logs an error). However, for a strengthened verification, requiring `self.spec_drop_safe()` as a precondition on `exit()` would prove that no thread can exit while holding mutexes — elevating the runtime error log to a compile-time proof obligation.
   - **Suggested Fix:** Consider adding `self.spec_drop_safe()` as a precondition to `exit()` if the project wants to enforce this invariant. Note: this would be *stronger* than the original behavior and would require all callers to prove mutex release before exit.

3. **Proof lemmas are trivially discharged**
   - **Location:** running.proof.rs, all lemmas
   - **Description:** All 24 proof lemmas have empty bodies, meaning the SMT solver can discharge them automatically from the spec definitions. While this is correct and demonstrates good spec design (properties follow directly from definitions), it means the proofs don't exercise deep reasoning. The lemmas serve primarily as documentation of proven properties rather than challenging verification obligations.
   - **Suggested Fix:** No action needed — trivially provable lemmas are a positive sign. Consider adding more complex composite lemmas if deeper properties are desired (e.g., "a thread that acquires N mutexes and releases all N is drop-safe again").

4. **Boundary model cross-module verification obligations are comments-only**
   - **Location:** `ReadyThread::from_state`, `SleepingThread::from_state`, `ZombieThread::from_state` in running.rs
   - **Description:** The "CROSS-MODULE-CHECK" comments document that the boundary model postconditions must be confirmed against the real implementations when those modules are independently verified. However, there is no automated mechanism to enforce this check. If the real `ReadyThread::from_state` has weaker postconditions than assumed, the verification would be unsound.
   - **Suggested Fix:** Consider creating a cross-module verification manifest or test that checks boundary model assumptions against real module specs when both are available.

## Positive Observations

- **Complete function coverage:** All 10 public functions from the original `RunningThread` are accounted for — 8 fully verified, 1 omitted with documented justification (`join_cond`), and 1 marked `#[verifier::external]` with clear trust obligations (`thread_state_mut`).
- **Zero `assume` or `external_body`:** No unjustified trust in the core verification. The only escape hatch (`#[verifier::external]` on `thread_state_mut`) is well-documented with intended postconditions and justified by a Verus language limitation.
- **Clean spec/proof/exec split:** The three-file organization is exemplary — specs define the abstract model, proofs establish properties, and exec code implements verified operations.
- **Rich mutex accounting:** The ghost set model with non-interference (`forall|a: int| a != address@ ==> ...`) faithfully captures `BTreeMap` per-key semantics. The well-formedness invariant tying `locked_mutex_count` to `locked_mutex_set.len()` is a clean protocol model.
- **Drop safety tracking:** The propagation of `spec_drop_safe()` through all transitions, combined with the proven equivalence to `check_drop_safe()` in ThreadState, provides end-to-end assurance of the mutex leak detection mechanism.
- **Well-documented trust boundary:** The module-level doc comments enumerate every modeling decision (Box transparency, SystemTime → `Option<int>`, etc.), every omission (ContextInformation, Condvar, FpuState), and every trust assumption (T1: no double-lock, T2: release-what-you-hold). This is best-practice documentation for formal verification.
- **Boundary type design:** Modeling sibling types (SleepingThread, ReadyThread, ZombieThread) inline with cross-module verification obligations documented per-constructor is a clean approach to modular verification.
- **Verification passes cleanly:** 41 verified obligations, 0 errors, in 4 seconds.

## Summary

This is a high-quality Verus verification of `RunningThread` that covers all essential correctness properties: thread identity immutability, state transition correctness, well-formedness preservation, mutex accounting with per-address non-interference, and drop safety propagation. The verification model makes reasonable abstractions (Box transparency, raw pointer omission, sync boundary elision) and documents all trust assumptions clearly.

The main areas for improvement are minor: (1) an asymmetry in per-address mutex preservation between `exit()` and the other transitions, (2) the `take_mutex_guard` API strengthening from `Option<MutexGuard>` to unit with a precondition, and (3) the reliance on comments rather than automation for cross-module boundary verification. None of these compromise the soundness of what *is* verified — they represent opportunities to expand coverage or tighten the specification.

The grade of A- reflects excellent verification quality with minor completeness gaps in an otherwise thorough and well-organized verification effort.
