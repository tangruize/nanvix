# Bug Report: Semaphore `up()` Overflow — Silent Resource Counter Wraparound via `fetch_add`

> **Severity:** MEDIUM
> **Status:** Confirmed; not yet fixed in production code
> **Discovery Method:** Structural — Verus verification's arithmetic overflow rules forced the AI
> prover to add an explicit overflow guard precondition
> **Affected File:** [`src/kernel/src/pm/sync/semaphore.rs`](src/kernel/src/pm/sync/semaphore.rs) (line 155)
> **Verified File:** [`verus/split/kernel/pm/sync/semaphore.rs`](verus/split/kernel/pm/sync/semaphore.rs)
> **Category:** Synchronization Primitive Defect

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Trigger Scenario and Impact](#3-trigger-scenario-and-impact)
4. [How Verus Verification Discovered This Bug](#4-how-verus-verification-discovered-this-bug)
5. [Proposed Fix](#5-proposed-fix)
6. [File Reference Index](#6-file-reference-index)

---

## 1. Executive Summary

The `Semaphore::up()` function uses `AtomicUsize::fetch_add(1, SeqCst)` to increment the
semaphore's value counter. When the counter reaches `usize::MAX`, the addition wraps silently
to 0. This transitions the semaphore from "maximum resources available" to "no resources
available," causing all subsequent `down()` callers to block indefinitely despite no actual
resource exhaustion.

The Verus verification model surfaced this by requiring `old(self)@.value < usize::MAX as nat`
as a precondition to `up()`, making the implicit no-overflow assumption explicit.

---

## 2. The Bug

### Root Cause

In [`src/kernel/src/pm/sync/semaphore.rs` (line 155)](src/kernel/src/pm/sync/semaphore.rs#L155):

```rust
pub unsafe fn up(&self) -> Result<(), Error> {
    self.value.fetch_add(1, Ordering::SeqCst);  // ← WRAPS ON OVERFLOW
    self.sleeping.notify_first().map(|_awakened| ())
}
```

`AtomicUsize::fetch_add` performs wrapping addition. There is no check that `value < usize::MAX`
before incrementing.

### The Wraparound

| Before `up()` | After `up()` | Meaning                                    |
|----------------|--------------|---------------------------------------------|
| `usize::MAX`  | `0`          | All resources appear exhausted              |
| `usize::MAX-1`| `usize::MAX` | Valid (one more resource available)          |

### Why This Is Dangerous for a Semaphore

A semaphore's value represents the count of available resources. Wraparound from
`usize::MAX → 0` is semantically catastrophic:

1. **Resource starvation:** All threads calling `down()` will block, believing no resources
   are available.
2. **Deadlock potential:** If the thread calling `up()` also needs to call `down()` later,
   the system deadlocks.
3. **Silent corruption:** Unlike a panic, the wraparound produces no error, no log message,
   and no observable failure until threads start hanging.

---

## 3. Trigger Scenario and Impact

### Triggering Conditions

- A semaphore's value reaches `usize::MAX` through repeated `up()` calls without matching
  `down()` calls.
- This could occur due to:
  - A resource leak where `up()` is called but the corresponding `down()` never occurs.
  - A programming error that calls `up()` in a loop without bound.

### Practical Likelihood

Low in normal operation (requires `usize::MAX` unmatched `up()` calls), but the bug represents
a **violated safety invariant**: the semaphore's value should always accurately reflect available
resources, and the `up()` operation should always increase (never decrease) the count.

### Impact

| Aspect              | Impact                                                    |
|---------------------|-----------------------------------------------------------|
| **Resource starvation** | All threads see 0 resources, block indefinitely       |
| **Deadlock**        | System hangs if critical threads are blocked              |
| **Silent failure**  | No error reported; diagnosis requires inspecting raw counter |

---

## 4. How Verus Verification Discovered This Bug

### The Verification Model

The Verus model in [`verus/split/kernel/pm/sync/semaphore.rs`](verus/split/kernel/pm/sync/semaphore.rs)
formalized `up()` with an explicit precondition:

```rust
pub fn up(&mut self, ctx: Ghost<CallerContext>)
    requires
        old(self).wf(),
        old(self)@.value < usize::MAX as nat,  // ← OVERFLOW GUARD
        ctx@.safe_for_up(),
    ensures
        self@.value == old(self)@.value + 1,
        self@.waiters == old(self)@.waiters,
        self.spec_is_available(),
        self.wf(),
    { ... }
```

### Discovery Mechanism

When the AI prover wrote `self.value = self.value + 1`, Verus's SMT solver required a proof
that the result does not overflow `usize`. The prover had to add the
`old(self)@.value < usize::MAX` precondition, which is exactly the condition the original
`fetch_add` call silently ignores.

The AI reviewer confirmed the finding:

> "Overflow guard on `up()`: The original `fetch_add(1, SeqCst)` can silently overflow; the
> verified model makes this an explicit precondition (`value < usize::MAX`), surfacing a real
> bug class."

### Classification

This bug was discovered **structurally** — the formal verification framework cannot express
wrapping arithmetic without explicit annotation, so the overflow was surfaced as a proof
obligation that the original code fails to satisfy.

---

## 5. Proposed Fix

### Option A: Saturating Addition (preferred)

```rust
pub unsafe fn up(&self) -> Result<(), Error> {
    let prev = self.value.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
        if v < usize::MAX { Some(v + 1) } else { None }
    });
    match prev {
        Ok(_) => self.sleeping.notify_first().map(|_| ()),
        Err(_) => {
            error!("semaphore value overflow");
            Err(Error::new(ErrorCode::ValueOverflow, "semaphore value overflow"))
        }
    }
}
```

### Option B: Debug Assert

```rust
pub unsafe fn up(&self) -> Result<(), Error> {
    let prev = self.value.fetch_add(1, Ordering::SeqCst);
    debug_assert!(prev < usize::MAX, "semaphore value overflow");
    self.sleeping.notify_first().map(|_| ())
}
```

Option A is preferred because it preserves the safety invariant in both debug and release builds.

---

## 6. File Reference Index

| File | Role |
|------|------|
| `src/kernel/src/pm/sync/semaphore.rs` | Original source (bug at line 155) |
| `verus/split/kernel/pm/sync/semaphore.rs` | Verus exec model with overflow precondition |
| `verus/split/kernel/pm/sync/semaphore.spec.rs` | Spec: SemaphoreView with value/waiters |
| `verus/split/kernel/pm/sync/semaphore.proof.rs` | Proof: waiter draining and round-trip lemmas |
| `verus-ai-history/reviews/semaphore/gemini_r1_a1.md` | AI review identifying the overflow divergence |
| `verus-ai-history/reviews/semaphore/claude_r2_a1.md` | AI review confirming the bug class |
