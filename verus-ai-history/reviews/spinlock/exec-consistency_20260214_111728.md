# Review: spinlock Exec Consistency (claude-opus-4.6)

## Grade: A

## Overview

Reviewed the exec consistency fixes applied to the Verus-verified spinlock model
against the original source at `src/kernel/src/pm/sync/spinlock.rs`.

**Original source:** 3 items — `Spinlock` struct (wrapping `AtomicBool`), `SpinlockGuard<'a>` struct, `Spinlock::new()`, `Spinlock::lock()`, `Drop for SpinlockGuard`.

**Verified code:** `Spinlock` struct (plain `bool` + `id` + `token_issued`), `LockToken` tracked ghost struct, `new()`, `try_lock()`, `lock()`, `unlock()`, `is_locked()`.

**Verification result:** 24 verified, 0 errors. No `assume`, `admit`, `external_body`, or `trusted` annotations.

## Issues Found

### Critical

- None.

### Minor

1. **`Spinlock` fields are `pub`** (spinlock.rs:112–121). The doc comment explains this is
   required by Verus's `View` trait, and the coding standard mandates private fields with
   getters/setters for runtime code. Since this is a verification model (not runtime code),
   this is acceptable, but the justification comment could note the Nanvix coding standard
   exception more explicitly.

2. **`lock()` precondition `!old(self)@.token_issued`** (spinlock.rs:239) is redundant with
   `old(self).inv()` + `old(self)@.is_unlocked()`, since `inv()` enforces
   `locked == token_issued` and `is_unlocked()` enforces `!locked`. This is not incorrect
   — the explicit precondition aids readability and makes the obligation visible to callers
   — but could be noted as intentionally redundant for documentation purposes.

3. **`try_lock` postcondition verbosity** (spinlock.rs:184–195). The 10-clause ensures
   block is thorough but could benefit from a brief grouping comment (success-path vs
   failure-path clauses). This is a style nit, not a correctness issue.

## Detailed Analysis

### 1. MISMATCH Functions — Properly Restored or Equivalence Documented?

**`Spinlock` struct:** The fix report documents `AtomicBool` → `bool` as a necessary Verus
limitation. The `id` and `token_issued` fields are verification-only additions. The
module-level "Verification Model" documentation (lines 22–37) thoroughly explains this.
**Verdict: Sound.**

**`SpinlockGuard` struct:** Replaced by `LockToken` tracked ghost token. The "Trust
Boundaries" documentation (lines 66–73) explains that Verus cannot reason about lifetimes
or `Drop`. The token carries a view snapshot binding it to the lock instance via `id`.
**Verdict: Sound.**

**`new()`:** The `id` parameter addition is documented under "Source Equivalence" (lines
139–146) and Trust Assumption T1 (lines 78–82). The `const fn` removal is explained.
Core logic (`AtomicBool::new(false)` ≡ `locked: false`) is identical.
**Verdict: Sound.**

**`lock()`:** Divergences (`&self` → `&mut self`, CAS loop → `try_lock()` delegation,
`SpinlockGuard` → `Tracked<LockToken>`) are all documented in the "Source Equivalence"
section (lines 226–234). The delegation to `try_lock()` is sound because `inv()` +
`is_unlocked()` guarantees success. The state transition `locked: false → true` is
identical to the original's `compare_exchange(false, true, Acquire, Relaxed)`.
**Verdict: Sound.**

### 2. MISSING Functions — Added with Proper Verification?

**`Drop for SpinlockGuard`:** Cannot be directly translated because Verus cannot reason
about `Drop` traits. Instead:
- `unlock()` models the drop behavior (`self.locked = false` ≡ `store(false, Release)`).
- `lemma_unlock_models_drop` (proof.rs:272–285) formally proves the equivalence.
- The lemma verifies that post-unlock state is `is_unlocked()`, matches `spec_new(id)`,
  and satisfies `inv()`.
**Verdict: Sound.** The equivalence proof is non-vacuous (requires `inv()` and
`is_locked()` as preconditions).

### 3. Equivalence Justifications — Are They Sound?

| Equivalence | Assessment |
|---|---|
| `AtomicBool` → `bool` | Sound. Sequential model captures state machine, atomicity out of scope. |
| `&self` → `&mut self` | Sound. Interior mutability via atomics cannot be modeled in Verus. |
| `SpinlockGuard`/`Drop` → `LockToken`/`unlock()` | Sound. Token isolation via `id` + formal proof. |
| `id` parameter in `new()` | Sound. Compensates for Verus's inability to reason about reference identity. Trust assumption T1 is clearly documented. |
| CAS loop → `try_lock()` | Sound. Sequential preconditions guarantee first-attempt success. |

All five equivalences are well-justified with clear rationale. The documentation is
thorough and honest about what is and isn't being verified.

### 4. Does Exec Code Faithfully Represent the Original?

The exec code captures the essential state machine protocol:
- `new()`: produces unlocked state ✓
- `lock()` → `try_lock()`: transitions unlocked → locked, produces token ✓
- `unlock()`: transitions locked → unlocked, consumes token ✓
- Round-trip invariant: `new → lock → unlock` restores initial state ✓

The state transitions are identical to the original. What is not modeled (concurrency,
atomicity, spin-wait, memory ordering, RAII) is explicitly documented as out of scope.

### 5. Does Verification Pass?

**Yes.** 24 verified, 0 errors. No trust annotations (`assume`, `admit`, `external_body`,
`trusted`) are used anywhere in the exec, spec, or proof files. All lemmas are
automatically discharged by Verus.

## Spec and Proof Quality

**Spec file** (spinlock.spec.rs):
- `SpinlockView` is well-designed with `nat` for abstract `id`.
- `inv()` is `closed spec fn` (good — prevents external reasoning about internals).
- `is_locked()`/`is_unlocked()` are `open spec fn` (good — allows clients to reason).
- `spec_new()` provides a reference point for state comparisons.

**Proof file** (spinlock.proof.rs):
- 17 lemmas organized into definitional properties and protocol properties.
- `lemma_lock_unlock_roundtrip` proves full protocol soundness.
- `lemma_token_instance_isolation` proves cross-instance token safety.
- `lemma_unlock_models_drop` bridges the RAII gap.
- `lemma_lock_precondition_prevents_deadlock` justifies the `is_unlocked()` precondition.
- All lemmas verify without hints (except `reveal(Spinlock::inv)` where needed).

## Summary

The exec consistency fixes are thorough and well-executed. Every divergence from the
original source is necessitated by fundamental Verus limitations (no atomics, no `Drop`,
no interior mutability, no lifetime-bound types), and each is documented with clear
equivalence justifications. The `lemma_unlock_models_drop` proof formally bridges the
most significant gap (RAII → explicit unlock). The verification passes cleanly with 24
verified items and zero trust annotations. The documentation is exemplary — the module-level
comments provide a complete picture of what is verified, what is out of scope, and what
trust assumptions exist. Grade reflects minor style observations that do not affect
correctness or completeness.
