# Review: semaphore (claude-opus-4.6)

## Grade: B

## Verification Result

29 verified, 0 errors. No `assume`, `external_body`, or `trusted` annotations in executable or proof code.

## Issues Found

### Critical

_None._

### High

- **H1: `waiters` field is dead state — never mutated by any exec function**
  - Priority: High
  - Location: `Semaphore` struct and all exec functions (`semaphore.rs`)
  - Description: The `waiters` field is declared in the struct and referenced in `wf()` (e.g., `self@.waiters > 0 ==> self@.value == 0`), but no exec function (`new`, `down`, `try_down`, `up`) ever modifies `waiters`. It is initialized to 0 in `new()` and stays 0 forever. This means the `wf()` clause about waiters is vacuously true and never exercised. The waiter-tracking aspect of the spec is dead — it models nothing. This undermines the claim that the semaphore protocol is verified, since blocking/waking (the core concurrency mechanism) is entirely absent.
  - Suggested Fix: Either (a) add exec-level `sleep()`/`wake()` helper functions that increment/decrement `waiters` with appropriate pre/postconditions (matching the `Condvar::wait`/`notify_first` behavior), or (b) remove `waiters` from the exec struct and model it purely as ghost state in the spec with explicit state transition lemmas that prove the waiter invariant is maintained across sleep/wake transitions.

- **H2: `up()` precondition `old(self)@.waiters == 0` is overly restrictive**
  - Priority: High
  - Location: `up()` precondition (`semaphore.rs:245`)
  - Description: The original `up()` can be called at any time, including when threads are waiting (this is the primary use case — `up()` wakes a blocked thread via `notify_first()`). The verified model requires `waiters == 0`, which makes it impossible to reason about the wake-up path. Since `waiters` is always 0 (see H1), this precondition is trivially satisfied but renders the model unable to express the semaphore's core usage pattern: thread A blocks in `down()`, thread B calls `up()` to wake it.
  - Suggested Fix: Remove the `waiters == 0` precondition from `up()`. If waiter modeling is added (per H1), `up()` should be allowed to decrement `waiters` and increment `value`, or increment `value` with a postcondition that if there were waiters, one is now runnable.

- **H3: Blocking path of `down()` entirely unverified**
  - Priority: High
  - Location: `down()` (`semaphore.rs:183-194`)
  - Description: The original `down()` contains a `loop` that calls `fetch_update()` and, if the value is 0, sleeps on a `Condvar` and retries. This is the central correctness-critical path of a semaphore. The verified `down()` has a precondition `spec_is_available()` (value > 0) and performs a single decrement — it only models the instant-success case. The sleep-retry loop, the interaction with `Condvar`, and the guarantee that a blocked thread eventually succeeds (given eventual `up()`) are all unverified. The documentation acknowledges this as "out of scope," but it represents the majority of the semaphore's correctness surface.
  - Suggested Fix: Model the blocking case explicitly. At minimum, add a `down_blocking()` function that takes a semaphore with `value == 0`, increments `waiters`, and produces a "waiting" state. Then add a `wake()` transition (triggered by `up()`) that decrements `waiters` and completes the acquisition. This would verify the sleep/wake protocol without requiring full concurrency reasoning.

- **H4: Error handling completely elided**
  - Priority: High
  - Location: `down()`, `try_down()`, `up()` return types (`semaphore.rs`)
  - Description: The original functions return `Result<(), SleepError>`, `Result<(), Error>`, and `Result<(), Error>` respectively. The verified `down()` returns `()`, `try_down()` returns `bool`, and `up()` returns `()`. This means error propagation correctness is not verified. In particular: (1) `down()` can fail with `SleepError` from `Condvar::wait()` — the verified model cannot fail, (2) `try_down()` should return `ErrorCode::TryAgain` on failure — the verified model returns `false`, (3) `up()` can fail from `notify_first()` — the verified model cannot fail. The error paths are part of the original contract and callers depend on specific error codes.
  - Suggested Fix: Model return types as `Result`-like enums (or Verus `Result` types) matching the original signatures. For `try_down()`, ensure the failure case produces an error view equivalent to `ErrorCode::TryAgain`. For `down()` and `up()`, model the `Condvar` error propagation, even if the condvar itself is assumed correct.

### Medium

- **M1: Proof lemmas are mostly trivial definitional unfolding**
  - Priority: Medium
  - Location: All 23 proof lemmas (`semaphore.proof.rs`)
  - Description: The vast majority of the proof lemmas construct `SemaphoreView` structs with specific constant values and assert obvious arithmetic facts. For example, `lemma_binary_semaphore_mutual_exclusion()` just asserts that `SemaphoreView { value: 1, ... }.value == 1` and `SemaphoreView { value: 0, ... }.value == 0`. Similarly, `lemma_resource_conservation()` asserts `initial == current + acquired` given `current == initial - acquired` — pure arithmetic. These lemmas don't actually reason about the exec functions or state transitions; they reason about manually constructed views. The "Protocol Properties" section is mislabeled — these are arithmetic tautologies, not protocol proofs.
  - Suggested Fix: Replace view-construction lemmas with lemmas that reason about actual function pre/postconditions. For example: "Given a well-formed semaphore `s` with `s.value == n`, calling `down()` then `up()` yields a semaphore with `s.value == n`" — proved by chaining the ensures clauses of `down()` and `up()`. Prove inductive properties over sequences of operations rather than asserting facts about constant struct literals.

- **M2: `try_down()` return type diverges from original without justification for correctness**
  - Priority: Medium
  - Location: `try_down()` return type (`semaphore.rs:207`)
  - Description: The original `try_down()` returns `Result<(), Error>` where the error contains `ErrorCode::TryAgain`. The verified version returns `bool`. While this is documented as matching the "mutex `try_lock()` pattern for consistency," it means the verification doesn't prove that the correct error code is returned on failure. A caller might depend on receiving `TryAgain` specifically (vs. another error code), and this property is not captured.
  - Suggested Fix: Either use a Result-like return type that distinguishes `TryAgain` from other errors, or add a spec-level comment explicitly mapping `false` → `ErrorCode::TryAgain` with a justification that no other error codes are possible.

- **M3: No modular spec for composing with condvar verification**
  - Priority: Medium
  - Location: Spec file (`semaphore.spec.rs`)
  - Description: The documentation states "The sleeping/waking protocol via Condvar is assumed correct per the separately verified condvar module." However, there is no formal interface (e.g., a trait or spec function) that connects the semaphore spec to the condvar spec. The condvar module's guarantees are not imported or referenced. This makes the trust boundary informal — there is no machine-checked verification that the semaphore's assumptions about condvar actually match the condvar module's guarantees.
  - Suggested Fix: Define an interface spec (e.g., `spec fn condvar_guarantees_wake_after_notify()`) that the condvar module provides and the semaphore module requires. This creates a machine-checked trust boundary between modules.

### Low

- **L1: Extra functions `get_value()` and `is_available()` not in original**
  - Priority: Low
  - Location: `semaphore.rs:261-284`
  - Description: These functions do not exist in the original source. While they are benign helper functions with correct specs, they extend the API surface beyond the original. This is acceptable for verification convenience but should be noted for equivalence tracking.
  - Suggested Fix: No action needed. Optionally add a note in the API Mapping table marking these as "verification-only helpers."

- **L2: Struct fields are `pub` with documented exception**
  - Priority: Low
  - Location: `Semaphore` struct fields (`semaphore.rs:139-142`)
  - Description: Nanvix coding standards require private fields with getter/setter access. The fields are public due to Verus constraints for `pub open spec fn` access. This is documented but worth noting for standards compliance.
  - Suggested Fix: No action needed given Verus tooling constraints. Already documented.

## Positive Observations

- **Clean soundness**: No `assume`, `external_body`, or `trusted` annotations anywhere in the verification. All 29 verification conditions are discharged by the solver without escape hatches.
- **Excellent documentation**: The module-level documentation is unusually thorough, with explicit sections on verification scope, API mapping, API divergence, trust boundaries, trust assumptions, and a refinement argument connecting the sequential model to the concurrent implementation. This level of documentation is exemplary.
- **Correct spec/proof/exec separation**: The three-file split is clean. Spec functions are `open spec fn` in the spec file, proof lemmas are in the proof file, and exec code is in the main file. No cross-contamination.
- **Well-formedness invariant**: The `wf()` predicate correctly establishes the relationship between concrete and abstract state, and is correctly required/ensured by all functions.
- **Overflow protection**: The `up()` function has an explicit `value < usize::MAX` precondition, which captures a real correctness concern — the original `fetch_add(1, SeqCst)` can silently overflow in release mode.
- **`try_down()` both-path specification**: The `try_down()` ensures clause correctly specifies both the success path (value decremented) and the failure path (state unchanged), with the result mapped to `spec_is_available()`.

## Summary

The semaphore verification is well-structured, cleanly documented, and soundly verified (no escape hatches). It correctly verifies the arithmetic core of semaphore operations: incrementing, decrementing, overflow protection, and `try_down` both-path behavior. The three-file split is clean and the documentation is exemplary in its honesty about scope limitations.

However, the verification depth is limited. The model is essentially a verified counter — it proves that incrementing and decrementing a `usize` works correctly, which is not the hard part of a semaphore. The hard parts — blocking when the count is zero, waking blocked threads on `up()`, the interaction with `Condvar`, error propagation, and the loop-retry protocol — are all unverified. The `waiters` field, which was presumably introduced to model blocking, is dead state that no exec function ever modifies. The proof lemmas, while numerous (23), are almost entirely arithmetic tautologies about manually constructed structs rather than meaningful protocol properties.

**Recommendations for improvement (priority order):**
1. Model the blocking/waking protocol with explicit sleep/wake state transitions.
2. Remove the overly restrictive `waiters == 0` precondition from `up()`.
3. Replace trivial arithmetic lemmas with lemmas that chain function pre/postconditions.
4. Model error return types to match the original API contracts.
5. Define a formal interface between semaphore and condvar specs.
