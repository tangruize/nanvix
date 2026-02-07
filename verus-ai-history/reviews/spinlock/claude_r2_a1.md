# Review: spinlock (claude-opus-4.6)

## Grade: A-

## Verification Status

20 verified, 0 errors. No `assume` statements. One `external_body` (on `lock()`, justified).
No cheating patterns detected.

## Issues Found

### Critical

None.

### High

None.

### Medium

- **M1: `wf()` invariant is one-directional; missing reachability invariant**
  - **Location:** spec (`spinlock.spec.rs:80-82`)
  - **Description:** `wf()` is `!self.locked ==> !self.token_issued()`, which prevents
    "unlocked with token outstanding" but permits "locked without token outstanding"
    (`locked=true, token_issued=false`). In all states reachable from `new()` via the
    API (`new → lock/try_lock → unlock → ...`), the stronger biconditional
    `self.locked == self.token_issued()` holds. The current wf() allows the state
    `(locked=true, token_issued=false)` which is only reachable by direct struct
    construction (fields are `pub`) or via the `try_lock()` failure path on a
    non-API-constructed spinlock. While the specs are still sound (no incorrect proof
    can be derived), the weaker invariant means callers reasoning about reachable states
    must track the biconditional themselves rather than relying on `wf()`.
  - **Suggested Fix:** Strengthen `wf()` to `self.locked == self.token_issued()`, or add
    a separate `reachable()` predicate that implies the biconditional. Verify that all
    exec functions still verify under the stronger invariant (they should, since all
    transitions maintain `locked == token_issued`). Alternatively, add a proof lemma
    `lemma_reachable_locked_iff_token_issued` documenting this property even if `wf()`
    is not changed.

### Low

- **L1: `LockToken` forgeable in proof mode**
  - **Location:** spec (`spinlock.spec.rs:50-53`)
  - **Description:** `LockToken` has `pub ghost view`, allowing proof-mode code within
    the same crate to forge tokens via direct construction:
    `let tracked forged = LockToken { view: SpinlockView { locked: true, id: x, token_issued: true } };`
    The `token_issued` ghost tracking mitigates this substantially — a forged token can
    only be used to call `unlock()` on a spinlock that already has `token_issued == true`,
    meaning a real acquisition must have occurred. However, the forged token could be used
    *instead of* the real token, allowing the real token to be used elsewhere (e.g., double
    unlock of two different locks that share the same state).
  - **Suggested Fix:** This follows the established `KernelRedZoneGhost` pattern in the
    codebase and is documented in the Soundness section (spec.rs:46-49). No action
    required unless the project moves to a stricter token model with private fields.

- **L2: Proof lemmas remain trivially auto-discharged**
  - **Location:** proof (`spinlock.proof.rs`)
  - **Description:** All 15 proof lemmas have empty bodies and are automatically discharged
    by Verus. While parameterized over arbitrary inputs (not just constants), they expand
    to simple boolean tautologies. For example, `lemma_try_lock_unlocked_succeeds` with
    requires `pre.spec_is_unlocked()` concludes `!pre.locked` — which is the definition
    of `spec_is_unlocked`. Similarly, `lemma_lock_unlock_roundtrip` constructs concrete
    states and asserts properties that are immediate from the definitions.
  - **Suggested Fix:** Inherent to a single-boolean state machine. The lemmas serve as
    executable documentation of the protocol properties and as regression tests for spec
    changes. No action needed.

- **L3: Token affinity (not linearity) allows token leaks**
  - **Location:** exec (`spinlock.rs:202`), spec (`spinlock.spec.rs:50`)
  - **Description:** Verus `tracked` values are affine (can be dropped) not linear (must
    be consumed). A caller can `lock()` and drop the returned `LockToken` without calling
    `unlock()`, leaving the spinlock permanently locked. This mirrors `mem::forget` on a
    real `SpinlockGuard` — a known Rust limitation (see `std::mem::forget` is safe).
    The `token_issued` ghost field remains `true`, preventing further `lock()` or
    `try_lock()` calls (both require `!token_issued`), so the leaked state is at least
    detectable at the spec level.
  - **Suggested Fix:** Inherent Verus limitation. Documented in Trust Boundaries section
    (spinlock.rs:57-64). No action needed.

- **L4: `try_lock()` failure path unreachable from well-constructed spinlocks**
  - **Location:** exec (`spinlock.rs:158-181`)
  - **Description:** `try_lock()` requires `!old(self).token_issued()`. In all states
    reachable from `new()`, `!token_issued ==> !locked` (the biconditional from M1). This
    means `try_lock()` always succeeds on API-reachable states, and the failure path
    (lines 178-179) is only exercisable on states constructed by direct field access
    (e.g., `Spinlock { locked: true, ..., token_issued: Ghost(false) }`). The failure
    path specs are still correct and verified, but they verify dead code relative to the
    intended usage.
  - **Suggested Fix:** Document in `try_lock()` that the failure path exists for
    completeness but is unreachable from `new()`-constructed spinlocks. Alternatively,
    if M1 is fixed (strengthening wf), this becomes explicit: `wf()` + `!token_issued`
    would directly imply `!locked`, making the success path provably guaranteed.

## Positive Observations

- **Ghost instance identity resolves prior token interchangeability issue.** The `id:
  Ghost<nat>` field in `Spinlock` and `id: nat` in `SpinlockView` bind tokens to specific
  lock instances. `lemma_token_instance_isolation` (proof.rs:173-184) formally proves that
  tokens from different instances cannot satisfy each other's unlock preconditions. This
  was the primary architectural concern from the r1_a3 review (N1) and is now fully
  addressed.

- **`token_issued` ghost tracking provides meaningful state machine enforcement.** The
  combination of `token_issued` in the view, the `wf()` invariant, and the
  `requires !old(self).token_issued()` on `lock()`/`try_lock()` creates a real
  protocol-level guarantee: no double-locking, no spurious unlocking. This goes beyond
  documentation into machine-checked enforcement.

- **Single justified `external_body`.** Only `lock()` uses `external_body`, with thorough
  justification (spin-wait termination depends on concurrent unlock, atomic CAS cannot be
  modeled). The preconditions and postconditions are sound for the sequential model.

- **Excellent documentation.** The module header (spinlock.rs:1-74) provides comprehensive
  coverage of: verified properties, verification model, scope, API divergence, trust
  boundaries, and trust assumptions. This is exemplary documentation for a verified module.

- **Clean spec/proof/exec separation.** Spec types and functions in `spinlock.spec.rs`,
  proof lemmas in `spinlock.proof.rs`, exec code in `spinlock.rs`. Each file has a clear
  purpose with no mixing of concerns.

- **Protocol round-trip proven.** `lemma_lock_unlock_roundtrip` (proof.rs:85-104) proves
  that lock-then-unlock restores the original state with identity and wf() preserved
  through the full cycle. `lemma_unlocked_eq_new_view` (proof.rs:107-113) connects
  unlocked states back to the `new()` postcondition.

- **Trust assumption T1 (ID uniqueness) is clearly documented.** The module header
  (spinlock.rs:69-73) explicitly states that callers must provide unique ghost IDs and
  explains the consequence of violation. This is honest and helpful.

## Summary

The spinlock verification is a well-executed sequential model of a spin-wait mutual exclusion
primitive. It verifies the state machine protocol (lock/unlock transitions, token-based
release obligations, instance isolation via ghost IDs) while clearly delineating what is out
of scope (concurrency, atomicity, liveness, Drop semantics).

The tracked `LockToken` mechanism with ghost `id` and `token_issued` fields provides genuine
machine-checked guarantees: every unlock requires a prior lock, tokens are instance-bound,
and double-locking is prevented. The single `external_body` on `lock()` is well-justified
and its postconditions are sound.

The primary remaining issue (M1) is that the `wf()` invariant is weaker than the actual
reachable-state invariant — it permits `(locked=true, token_issued=false)` which is
unreachable from `new()`. Strengthening this would tighten the model and make `try_lock()`'s
success path provably guaranteed. The low-priority issues (token forgeability, trivial
lemmas, affine token leaks, dead failure path) are inherent to the verification tool and
problem domain.

Overall, this is a high-quality verification with thorough documentation, proper trust
boundary delineation, and meaningful machine-checked properties. The grade of A- reflects
the minor `wf()` gap and the inherent limitations of sequential modeling for a concurrency
primitive.
