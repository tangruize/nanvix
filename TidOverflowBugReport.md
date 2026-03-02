# Bug Report: ThreadManager TID Overflow — Silent Wraparound Producing Negative Thread Identifiers

> **Severity:** MEDIUM-HIGH
> **Status:** Fixed in commit [`4bea8929f`](https://github.com/search?q=4bea8929f) (2026-02-19)
> **Discovery Method:** Structural — Verus verification's arithmetic overflow rules forced the AI
> prover to add an explicit overflow precondition, surfacing a latent bug in the original code
> **Affected File:** [`src/kernel/src/pm/thread/mod.rs`](src/kernel/src/pm/thread/mod.rs) (line 202)
> **Verified File:** [`verus/split/kernel/pm/thread/mod.rs`](verus/split/kernel/pm/thread/mod.rs) (lines 389–392)
> **Category:** Scheduler Correctness Violation

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Trigger Scenario and Impact](#3-trigger-scenario-and-impact)
4. [How Verus Verification Discovered This Bug](#4-how-verus-verification-discovered-this-bug)
5. [The Fix](#5-the-fix)
6. [File Reference Index](#6-file-reference-index)

---

## 1. Executive Summary

The `ThreadManager::create_thread()` function incremented the next thread identifier using
unchecked arithmetic: `ThreadIdentifier::from(<i32>::from(self.next_id) + 1)`. When `next_id`
reaches `i32::MAX` (2,147,483,647), the addition overflows. In debug mode, this panics; in
release mode, it wraps silently to `i32::MIN` (-2,147,483,648), producing a negative thread
identifier.

The Verus verification model made this implicit no-overflow assumption explicit via a
precondition `old(self)@.next_id < i32::MAX as int`, directly surfacing the latent bug.
The fix was committed to mainline as `4bea8929f`, introducing `checked_add` with proper
error reporting.

---

## 2. The Bug

### Root Cause

In the original [`src/kernel/src/pm/thread/mod.rs` (line 202)](src/kernel/src/pm/thread/mod.rs#L202):

```rust
pub fn create_thread(&mut self, ...) -> ReadyThread {
    let id: ThreadIdentifier = self.next_id;
    self.next_id = ThreadIdentifier::from(<i32>::from(self.next_id) + 1);  // ← UNCHECKED
    ReadyThread::new(id, ...)
}
```

The `+ 1` operation on `i32` has no overflow guard. When `self.next_id` is `i32::MAX`:

| Mode    | Behavior                                                     |
|---------|--------------------------------------------------------------|
| Debug   | Panic: `attempt to add with overflow`                        |
| Release | Silent wrap to `i32::MIN` → negative TID assigned to thread  |

### Why Negative TIDs Are Dangerous

1. **Identity collision:** A negative TID may collide with valid identifiers in other lookup tables
   or sentinel values, corrupting process-thread mappings.
2. **Monotonicity violation:** The thread manager relies on TIDs being strictly monotonic for
   uniqueness guarantees. Wraparound breaks this invariant, potentially allowing two distinct
   threads to share the same TID.
3. **Downstream confusion:** Any code using `tid >= 0` as a validity check will misclassify the
   thread. The kernel thread has TID 0; a negative TID may pass or fail validity checks
   unpredictably.

### Same Bug Existed for Process Identifiers

The identical pattern existed in `ProcessManager` for PID allocation. Both were fixed in the
same commit.

---

## 3. Trigger Scenario and Impact

### Triggering Conditions

- A long-running Nanvix system (or fork-bomb scenario) creates 2,147,483,647 threads over the
  lifetime of the kernel.
- Thread identifiers are never recycled (documented as FIXME #1440).
- After `i32::MAX` threads, the next `create_thread` call wraps.

### Practical Likelihood

Low in normal operation (requires billions of thread creations), but:

- **Fork bombs** can exhaust the identifier space rapidly.
- **Embedded/microkernel deployments** may run for months without reboot.
- The bug is a **correctness violation** regardless of practical likelihood — the invariant
  "all TIDs are unique and positive" is silently broken.

### Impact

| Aspect              | Impact                                                      |
|---------------------|-------------------------------------------------------------|
| **Data corruption** | Thread identity table corrupted with negative/duplicate TIDs |
| **Kernel stability**| Unpredictable behavior when looking up threads by TID        |
| **Security**        | Potential privilege confusion if TIDs are used for access control |

---

## 4. How Verus Verification Discovered This Bug

### The Verification Model

The Verus model in [`verus/split/kernel/pm/thread/mod.rs`](verus/split/kernel/pm/thread/mod.rs)
formalized the `create_thread` function with an explicit precondition:

```rust
pub fn create_thread(&mut self, ...) -> (result: ReadyThread)
    requires
        old(self).wf(),
        old(self)@.next_id < i32::MAX as int,  // ← OVERFLOW GUARD
    ensures
        result.spec_id() == old(self).spec_next_id(),
        self.spec_next_id() == old(self).spec_next_id() + 1,
        self.wf(),
    { ... }
```

### Discovery Mechanism

Verus requires all arithmetic to be proven overflow-free. When the AI prover wrote
`self.next_id.into_i32() + 1`, Verus's SMT solver demanded a proof that the result fits in
`i32`. The prover had to add `old(self)@.next_id < i32::MAX as int` as a precondition —
this is the exact condition the original code failed to check.

The AI reviewer then noted in the review:

> "Overflow precondition catches latent bug: The original `create_thread` does not check for
> integer overflow when incrementing `next_id`. The verification model's precondition
> `old(self).next_id.value < i32::MAX` formalizes this assumption, effectively identifying a
> latent overflow bug in the original code."

### Classification

This bug was discovered **structurally** — not by running a test case, but because the formal
verification framework requires explicit reasoning about every arithmetic operation. The bug
would be extremely difficult to find through testing alone, as it requires exactly `i32::MAX`
thread creations to trigger.

---

## 5. The Fix

### Mainline Fix (commit `4bea8929f`)

The fix introduced `try_next_tid()` with `checked_add`:

```rust
pub(crate) fn try_next_tid(&self) -> Result<(ThreadIdentifier, ThreadIdentifier), Error> {
    let id: ThreadIdentifier = self.next_id;
    let raw_id: i32 = <i32>::from(self.next_id);
    let next_raw_id: i32 = match raw_id.checked_add(1) {
        Some(val) => val,
        None => {
            let reason: &str = "thread identifier overflow";
            error!("{reason} (next_id={raw_id:?})");
            return Err(Error::new(ErrorCode::ValueOverflow, reason));
        },
    };
    Ok((id, ThreadIdentifier::from(next_raw_id)))
}
```

The `create_thread` method was refactored to take a pre-allocated identifier, separating the
fallible ID allocation from the infallible thread construction.

---

## 6. File Reference Index

| File | Role |
|------|------|
| `src/kernel/src/pm/thread/mod.rs` | Original source (bug at line 202, fixed) |
| `verus/split/kernel/pm/thread/mod.rs` | Verus verification model with overflow precondition |
| `verus/split/kernel/pm/thread/mod.spec.rs` | Spec: ThreadManagerView with wf() invariant |
| `verus/split/kernel/pm/thread/mod.proof.rs` | Proof: TID monotonicity and uniqueness lemmas |
| `verus-ai-history/reviews/thread_manager/claude_r1_a1.md` | AI review identifying the bug |
