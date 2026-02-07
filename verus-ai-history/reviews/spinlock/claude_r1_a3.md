# Re-Review (Round 3): spinlock (claude-opus-4.6)

## Grade: A-

## Previous Review Summary

The previous review (A-) had 2 remaining issues, both marked as accepted limitations:
1. **H1 residual (Medium):** Lock-release obligation not enforced at proof level.
2. **L2 residual (Low):** Proof lemmas remain trivially provable.

The previous review explicitly stated "No further action expected" for both issues. The
prover chose to address them anyway. This re-review evaluates whether the changes are
genuine improvements or introduce new problems.

## Verification Status

17 verified, 0 errors. No `assume` statements. One `external_body` (on `lock()`, justified).
Verification count increased from 16 to 17 (net: replaced 2 old lemmas with 3 new ones).
No cheating patterns.

## Issue-by-Issue Disposition

### H1 residual: Lock-release obligation — **Substantially Improved (downgraded to Low)**

**What was done:**
The prover introduced a `pub tracked struct LockToken` (spec.rs:40-43) and threaded it
through the API:
- `lock()` returns `Tracked<LockToken>` (spinlock.rs:155)
- `try_lock()` returns `(bool, Tracked<Option<LockToken>>)` (spinlock.rs:118)
- `unlock()` consumes `Tracked(token): Tracked<LockToken>` (spinlock.rs:180)
- `unlock()` requires `token.view == old(self)@` (spinlock.rs:183)

This follows the `KernelRedZoneGhost` tracked-token pattern already established in this
codebase (verus/split/kernel/mm/kredzone.rs).

**Critical analysis — what the token enforces:**

1. ✅ **Every `unlock()` requires a prior `lock()`.** Since `unlock()` requires a
   `Tracked<LockToken>` parameter, and tokens can only be legitimately produced by `lock()`
   or a successful `try_lock()`, callers cannot call `unlock()` without first acquiring.
   This is a real, machine-checked guarantee.

2. ✅ **Tokens cannot be duplicated.** Verus `tracked` values are linear — they cannot be
   `Copy`'d or cloned. A single token can only discharge one `unlock()`.

3. ✅ **Token view must match lock state.** The `requires token.view == old(self)@` clause
   binds the token to the current lock state. This prevents using a stale or unrelated
   token.

4. ❌ **Every `lock()` need not be followed by `unlock()`.** A caller can receive the
   token and drop it (Rust allows dropping any value; Verus `tracked` values are affine,
   not strictly linear). This mirrors the real-world possibility of `mem::forget` on a
   `SpinlockGuard`.

5. ⚠️ **Tokens are interchangeable between lock instances.** The `LockToken.view` is
   `SpinlockView { locked: true }` for *all* locked spinlocks. A caller with two spinlocks
   could `lock()` both and then pass spinlock A's token to spinlock B's `unlock()` — the
   `token.view == old(self)@` check passes because both views are `{ locked: true }`. The
   original `SpinlockGuard<'a>(&'a Spinlock)` binds to a specific instance via the
   lifetime reference; the verified `LockToken` does not carry instance identity.

6. ⚠️ **Token fields are `pub ghost`.** Proof-mode code within the same crate can forge a
   `LockToken` by constructing it directly:
   `let tracked forged = LockToken { view: SpinlockView { locked: true } };`
   The doc warns "Callers must not duplicate or forge tokens" (spec.rs:38-39), but this is
   a social contract, not a machine-enforced one. Making the field `pub(super) ghost` or
   adding a private `_phantom` field would prevent external forgery; however, this is
   consistent with the existing `KernelRedZoneGhost` pattern in the codebase which also
   uses `pub ghost` fields.

**Verdict:** This is a genuine, substantial improvement. The token mechanism provides a
real machine-checked guarantee (point 1-3) that did not exist before: spurious unlocks are
now impossible, and every unlock proves prior acquisition. The residual gaps (points 4-6)
are inherent to Verus's affine tracked model and the value-based (not identity-based)
Spinlock representation. Points 5-6 are new observations but Low severity — they require
adversarial proof-mode code or multi-lock scenarios outside the sequential model's primary
use case. Downgraded from Medium to Low.

### L2 residual: Proof lemmas trivially provable — **Marginally Improved (remains Low)**

**What was done:** Replaced 2 constant-based lemmas with 3 parameterized lemmas:
1. `lemma_new_then_try_lock_succeeds(s: &Spinlock)` — universally quantified over input.
2. `lemma_lock_token_valid_for_unlock(s: &Spinlock, token: &LockToken)` — involves
   `LockToken` type.
3. `lemma_lock_token_snapshot_is_locked(token: &LockToken)` — reasons about token state.

**Critical analysis:**
All three are still automatically discharged by Verus (empty proof bodies). The
parameterization is a cosmetic improvement — going from `Spinlock { locked: false }` to
an arbitrary `s` with `s@ == Spinlock::spec_new_view()` doesn't change the proof difficulty
(Verus expands the spec and sees `!false`). The `LockToken` lemmas (proof.rs:131-153) are
similarly trivial: given `token.view == s@` and `s.locked`, conclude `token.view.locked` —
this is one-step field access.

**Verdict:** The lemmas now involve the `LockToken` type, which makes them marginally more
useful as API documentation and for callers reasoning about tokens. But they remain
tautologies. For a single-boolean state machine, this is inherent. Remains Low.

## New Issues Check

### N1 (Low): Token interchangeability across Spinlock instances

**Location:** spec (`spinlock.spec.rs:40-43`), exec (`spinlock.rs:180-183`)

**Description:** As analyzed in H1 point 5 above, `LockToken` carries only
`SpinlockView { locked: bool }`, not an identifier for which `Spinlock` instance produced
it. Since all locked spinlocks produce tokens with `view == SpinlockView { locked: true }`,
tokens from different locks are interchangeable:

```
let mut a = Spinlock::new();
let mut b = Spinlock::new();
let token_a = a.lock();       // token_a.view == { locked: true }
let token_b = b.lock();       // token_b.view == { locked: true }
a.unlock(token_b);            // passes: token_b.view == a@ == { locked: true }
b.unlock(token_a);            // passes: token_a.view == b@ == { locked: true }
```

The original `SpinlockGuard<'a>(&'a Spinlock)` prevents this via the lifetime reference.

**Mitigation:** In the sequential `&mut self` model, multi-lock scenarios are limited by
Rust's borrow checker (though not eliminated — borrows are temporary, not held across calls).
Adding a ghost instance ID to both `Spinlock` and `LockToken` would fully resolve this, but
would add complexity beyond the current verification scope.

**Severity:** Low. Requires a multi-lock scenario and adversarial token passing. Does not
undermine the core single-lock protocol verification.

### N2 (Low): `LockToken` forgeable in proof mode

**Location:** spec (`spinlock.spec.rs:40-43`)

**Description:** As analyzed in H1 point 6, the `pub ghost view` field allows proof-mode
code to construct tokens without a corresponding lock acquisition. The `Soundness` section
documents this as a social contract. This follows the codebase convention established by
`KernelRedZoneGhost` (which also has `pub ghost` fields with a "do not forge" warning).

**Severity:** Low. Consistent with established codebase patterns. Requires deliberately
adversarial proof-mode code. Documented.

## Remaining Issues

### Low

- **N1: Token interchangeability.** `LockToken` lacks instance identity, making tokens
  from different locks interchangeable. Low severity — limited by sequential `&mut` model,
  affects only multi-lock scenarios.

- **N2: Token forgeable in proof mode.** `pub ghost` fields allow direct construction.
  Consistent with codebase patterns. Documented in Soundness section.

- **L2 (residual): Proof lemmas trivially provable.** Inherent to single-boolean state
  machine. Lemmas serve as executable API documentation.

## Positive Observations

- **Proactive improvement beyond review expectations.** The previous review marked both
  remaining issues as "accepted limitations" with "no further action expected." The prover
  implemented the tracked token mechanism anyway, which is the most architecturally
  significant change across all review rounds.

- **Follows established codebase patterns.** The `LockToken` design mirrors the
  `KernelRedZoneGhost` pattern in `verus/split/kernel/mm/kredzone.rs` — tracked struct with
  ghost view field, produced by creation functions, threaded through API calls. This is not
  an ad-hoc invention but an application of the project's existing idiom.

- **Real machine-checked guarantee added.** The token mechanism provides a genuine new
  property: `unlock()` cannot be called without a prior matching `lock()` or `try_lock()`.
  This was previously unenforced. The guarantee is one-directional (lock→token→unlock
  required; token-leak still possible) but meaningful — it prevents an entire class of
  bugs (spurious unlocks, double unlocks).

- **Clean API evolution.** The `try_lock()` return type `(bool, Tracked<Option<LockToken>>)`
  correctly models the two paths: success produces a token, failure produces none. The
  ensures clauses cover both cases with appropriate conditions. The `unlock()` token
  consumption is clean and well-documented.

- **Documentation accurately reflects mechanism.** The Trust Boundaries section (lines 42-46)
  correctly describes the tracked token approach, replacing the previous "callers must
  manually audit" language with a precise description of the enforcement mechanism.

- **17 verified, 0 errors.** Clean verification with no cheating. One justified
  `external_body`.

## Summary

The prover made a substantial architectural improvement by introducing a `LockToken` tracked
ghost token, going beyond what the previous review required. This is the most meaningful
change in the review cycle, transforming the lock-release obligation from a documentation-only
social contract into a partially machine-enforced type-level property.

**What the token enforces (machine-checked):**
- Every `unlock()` requires a token from a prior `lock()` or `try_lock()`.
- Tokens are linear (cannot be duplicated).
- Token state must match the lock state.

**What the token does not enforce (inherent limitations):**
- Tokens can be leaked (Verus tracked values are affine, not linear).
- Tokens lack instance identity (interchangeable between locks).
- Tokens are forgeable in proof mode (pub ghost fields).

These limitations are inherent to Verus's type model and consistent with established
codebase patterns. They are documented appropriately.

The three new Low issues (token interchangeability, token forgeability, trivial lemmas) are
design trade-offs rather than fixable oversights. The module is well-verified within the
constraints of its sequential model, well-documented, and follows project conventions.

**Grade remains A-.** The tracked token is a genuine improvement that closes the primary
gap from the previous review (lock-release obligation). The new Low issues it introduces
are inherent trade-offs, not regressions. The module does not reach A because the token
mechanism, while good, has known limitations that a more sophisticated design (instance IDs,
private fields) could address, and the proof lemmas remain trivially provable.
