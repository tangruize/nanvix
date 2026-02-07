# Review: semaphore (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

- **Location:** `View for Semaphore` (spec, line 324-326) and `wf()` (spec, line 115-118)
  - **Description:** The `View` implementation hardcodes `waiters: 0`, making the `wf()` invariant clause `(self@.waiters > 0 ==> self@.value == 0)` trivially satisfied for all exec-constructed views. This means exec-level `wf()` checks never actually enforce the waiter-value constraint — the constraint is only meaningful for manually constructed `SemaphoreView` values in proof lemmas. While this is an intentional design choice (documented in "Ghost State Architecture"), it creates a gap: the blocking protocol lemmas operate on abstract views disconnected from exec state, so no exec function's postcondition ever establishes a non-zero waiter count. The `down_or_block` WouldBlock postcondition says `self@ == old(self)@` (both with waiters=0), while `spec_down_or_block_ghost_view` for WouldBlock increments waiters — but this ghost view is never connected back to exec state.
  - **Suggested Fix:** This is inherent to the sequential model without ghost tokens. Document explicitly that exec-level `wf()` is weaker than spec-level `spec_wf()`. Consider adding a tracked ghost field or a separate ghost type that bridges the exec/spec gap for waiter tracking, if Verus supports it. Alternatively, add a "gap analysis" comment noting that the waiter invariant is only proven for the abstract state machine, not for exec-reachable states.

- **Location:** `up()` (exec, line 365-378) vs. original `up()` (original, line 154-157)
  - **Description:** The original `up()` returns `Result<(), Error>` because `self.sleeping.notify_first()` can fail. The verified `up()` returns `()` unconditionally. This drops the error path entirely. Trust assumption T5 documents this, but the consequence is significant: if `notify_first()` fails at runtime, the semaphore value has been incremented but no waiter is woken, potentially causing indefinite blocking. The spec-level `spec_wake` transition assumes the wake always succeeds, but the exec model doesn't even return a result type that could represent failure.
  - **Suggested Fix:** Change `up()` return type to a result-like enum (e.g., `UpOutcome { Notified, NoWaiters, NotifyFailed }`) to at least model the possibility of notification failure. Alternatively, add a spec-level postcondition noting the assumption that notification succeeds, with a proof obligation for callers.

### Medium

- **Location:** `down_or_block()` (exec, line 286-306) vs. original `down()` (original, line 83-101)
  - **Description:** The original `down()` returns `Result<(), SleepError>` — the `Condvar::wait()` call can fail (e.g., signal interruption), and the error propagates via `?`. The verified `down_or_block()` returns `DownOutcome` with only `Acquired` and `WouldBlock` variants. There is no `SleepError` variant. Trust assumption T6 documents this, but the error-then-retry path (`wait` fails, loop retries) is a real control flow path in the original that has no representation in the verified model.
  - **Suggested Fix:** Add a `WouldBlockError` or `SleepFailed` variant to `DownOutcome` to model the case where `Condvar::wait()` returns an error. This would allow proving that the semaphore state is unchanged on sleep failure.

- **Location:** `down_or_block()` postconditions (exec, line 291-298)
  - **Description:** The `WouldBlock` postcondition ensures `self@ == old(self)@`, which is correct at the exec level (no exec state changes). However, the corresponding ghost view function `spec_down_or_block_ghost_view(old(self)@, WouldBlock)` returns a view with `waiters + 1`, and there's an ensures clause for the Acquired case (`self@ == spec_down_or_block_ghost_view(...)`) but no equivalent for the WouldBlock case. The caller has no verified connection between the WouldBlock exec result and the ghost waiter increment.
  - **Suggested Fix:** Add an ensures clause: `result == DownOutcome::WouldBlock ==> Semaphore::spec_down_or_block_ghost_view(old(self)@, result).waiters == old(self)@.waiters + 1`. This at least exposes the ghost transition to callers for spec-level reasoning.

- **Location:** `spec_wake()` (spec, line 200-206)
  - **Description:** The `spec_wake` function has `recommends` (soft preconditions) for `view.waiters > 0` and `view.value == 1`, but no hard `requires`. If called with `view.value == 0` (e.g., before the `up()` that should precede it), the subtraction `view.value - 1` underflows to `nat::MAX` in Verus semantics. While `recommends` triggers warnings, it doesn't prevent misuse in proofs.
  - **Suggested Fix:** Consider strengthening `recommends` to a comment explaining why these aren't `requires` (if there's a reason), or add `requires` to prevent spec-level misuse. All current call sites satisfy these conditions, but future proof additions could inadvertently violate them.

- **Location:** Proof lemmas (proof, entire file)
  - **Description:** Many proof lemmas are trivial arithmetic identities that Verus's SMT solver handles automatically (e.g., `lemma_up_monotonic`: `(v + 1) > v`, `lemma_down_monotonic`: `(v - 1) < v`). These don't add verification strength — they're essentially documentation. The substantive lemmas (`lemma_all_waiters_eventually_served`, `lemma_up_wake_cycle`, `lemma_down_blocking_preserves_wf`) are valuable. The trivial ones add maintenance burden without verification benefit.
  - **Suggested Fix:** Consider tagging trivial lemmas as "documentation lemmas" vs. "structural lemmas" to distinguish those needed for proof propagation from those that exist for human readability.

### Low

- **Location:** `Semaphore` struct (exec, line 191-194)
  - **Description:** The `value` field is `pub` with a comment explaining this is a Verus constraint. Per Nanvix coding standards, struct fields should be private with getter/setter access. The `get_value()` helper exists but the field is directly accessed in specs via `self.value`.
  - **Suggested Fix:** This is a known Verus tooling limitation. No action needed beyond the existing documentation.

- **Location:** `get_value()` and `is_available()` (exec, lines 390-418)
  - **Description:** These are verification-only helpers not present in the original API. They're well-documented as such and provide spec-connected access. No functional concern, but they expand the verified API surface beyond the original.
  - **Suggested Fix:** None needed — properly documented.

- **Location:** Module documentation (exec, lines 1-161)
  - **Description:** The documentation is exceptionally thorough (API mapping table, trust assumptions, ghost state architecture, refinement argument, verification scope). Minor note: the "Refinement Argument" section (lines 149-160) provides an informal linearizability argument but notes it's not mechanized. This is honest but could be made more prominent as a key limitation.
  - **Suggested Fix:** Consider adding a one-line summary to the "Verification Scope" section: "The sequential-to-concurrent refinement (linearizability) is argued informally, not mechanically verified."

## Positive Observations

- **Zero assume/external_body/trusted annotations.** The entire verification (44 lemmas) is fully mechanized with no trust gaps in the core module. This is exemplary.
- **Excellent documentation.** The module documentation is among the best I've seen for verified code: it explicitly states what is and isn't verified, maps APIs between original and model, catalogs trust assumptions (T1-T6), and explains the ghost state architecture. A reader can quickly understand the verification's scope and limitations.
- **Comprehensive blocking protocol model.** The spec-level state machine for the sleep/wake protocol (`spec_down_blocking`, `spec_wake`, `spec_after_n_up_wake_cycles`) with the inductive `lemma_all_waiters_eventually_served` provides meaningful assurance about the protocol's correctness, even though it's at the spec level.
- **CallerContext ghost state.** Encoding the original `unsafe` preconditions (interrupts disabled, not kernel process, no held resources) as ghost preconditions is a clean design that preserves safety documentation in the verified model.
- **Overflow protection.** Trust assumption T1 is properly surfaced as a hard precondition on `up()` (`value < usize::MAX`), converting a silent runtime overflow in the original to an explicit verification obligation.
- **Well-structured split.** The spec/proof/exec separation is clean. Specs are purely declarative, proofs contain only lemmas, and exec code is minimal. The `include!` mechanism keeps them connected.
- **The `down_or_block` function** elegantly bridges the exec and spec layers, modeling both outcomes of the original `down()` loop in a single function call.

## Summary

This is a high-quality verification of a kernel semaphore's sequential state machine. The verification covers all original API functions, proves meaningful protocol properties (round-trips, resource conservation, mutual exclusion, waiter drainage), and is fully mechanized with no trust gaps in the module itself.

The primary limitation is the inherent disconnect between exec-level state (where `waiters` is always 0) and spec-level ghost state (where `waiters` tracks the blocking protocol). This means the blocking protocol proofs operate on an abstract state machine that is not directly connected to exec-reachable states. The limitation is well-documented but represents a real verification gap for the blocking path.

The secondary concern is dropped error paths: the original's `Result` return types for `down()` and `up()` encode real failure modes (`SleepError`, condvar notification failure) that are simplified away in the verified model. These are documented as trust assumptions but reduce the model's fidelity to the original implementation.

**Recommendations:**
1. Strengthen the exec-spec connection for waiter tracking (High).
2. Model the `up()` notification failure path (High).
3. Add a `SleepFailed` variant to `DownOutcome` (Medium).
4. Add explicit `requires` to `spec_wake` instead of `recommends` (Medium).
5. The trivial arithmetic lemmas could be pruned or tagged for clarity (Medium, low priority).
