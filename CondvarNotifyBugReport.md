# Bug Report: Condvar `notify_process()` — Documentation Claims "All Threads" but Wakes Only First

> **Severity:** LOW-MEDIUM
> **Status:** Confirmed; not yet fixed in production code
> **Discovery Method:** AI review — Verus model faithfully matched the implementation, and the AI
> reviewer noticed the documentation contradicts the code
> **Affected File:** [`src/kernel/src/pm/sync/condvar.rs`](src/kernel/src/pm/sync/condvar.rs)
> **Verified File:** [`verus/split/kernel/pm/sync/condvar.rs`](verus/split/kernel/pm/sync/condvar.rs)
> **Category:** Specification-Implementation Divergence

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Impact](#3-impact)
4. [How Verus Verification Discovered This Bug](#4-how-verus-verification-discovered-this-bug)
5. [Proposed Fix](#5-proposed-fix)
6. [File Reference Index](#6-file-reference-index)

---

## 1. Executive Summary

The `Condvar::notify_process()` function's documentation states "Wakes up all threads of a
process that are waiting on the target condition variable." However, the implementation only
wakes the **first** matching thread: it uses `iter().position()` to find the first entry with
a matching PID, then calls `remove(at)` to remove that single entry.

This is a specification-implementation divergence. Any caller relying on the documented behavior
(waking all threads of a process) will leave threads sleeping indefinitely.

---

## 2. The Bug

### The Documentation

In [`src/kernel/src/pm/sync/condvar.rs`](src/kernel/src/pm/sync/condvar.rs):

```rust
///
/// # Description
///
/// Wakes up all threads of a process that are waiting on the target condition variable.
///                ^^^
///                This is incorrect — only the FIRST matching thread is woken.
```

### The Implementation

```rust
pub unsafe fn notify_process(&self, pid: ProcessIdentifier) -> Result<(), Error> {
    // Find process.
    let idx: Option<usize> = self
        .inner
        .sleeping
        .borrow()
        .iter()
        .position(|&(p, _)| p == pid);  // ← Returns FIRST match only

    // Remove process from sleeping queue.
    if let Some(at) = idx {
        let (_notified_pid, tid) = self.inner.sleeping.borrow_mut().remove(at);  // ← Removes ONE entry
        // ...
        ProcessManager::wakeup(tid)?;  // ← Wakes ONE thread
    }
    // ...
}
```

### The Discrepancy

| Aspect | Documentation | Implementation |
|--------|--------------|----------------|
| Threads woken | All threads of the process | First thread of the process |
| Queue entries removed | All entries with matching PID | One entry with matching PID |
| Behavior | `notify_all` semantics | `notify_one` semantics |

---

## 3. Impact

### Scenario

If a process has 3 threads (T1, T2, T3) waiting on a condvar, and another thread calls
`notify_process(pid)`:

| Expected (per docs) | Actual |
|---------------------|--------|
| T1, T2, T3 all wake up | Only T1 wakes up |
| Queue is cleared of all entries for this PID | T2 and T3 remain sleeping |

### Consequence

- **Thread starvation:** Threads T2 and T3 remain sleeping indefinitely unless explicitly
  woken by subsequent calls.
- **Behavioral surprise:** Callers trusting the documentation will not issue additional
  `notify_process` calls, assuming all threads were already woken.
- **Hard to diagnose:** The misbehavior manifests as intermittent hangs that depend on how many
  threads of the same process are waiting.

### Severity Assessment

This is rated LOW-MEDIUM because:

- The code is **internally consistent** (it does what it does reliably).
- The bug is in the **documentation**, not the logic.
- However, any caller relying on the documented "all threads" behavior has a latent correctness
  issue.

---

## 4. How Verus Verification Discovered This Bug

### The Verification Model

The Verus verification model for `notify_process` faithfully matched the **implementation**
(first-match removal), not the documentation. The AI prover modeled `position()` as finding
the first matching entry and `remove(at)` as removing a single entry.

### Discovery by AI Reviewer

The AI reviewer compared the documentation against the model and noted:

> "The fix report correctly identifies that the original's doc comment says 'Wakes up all
> threads of a process' but the implementation only wakes the *first* matching thread. The
> model follows the implementation (correct), but the original source has a documentation bug."

And recommended:

> "File a documentation bug against `src/kernel/src/pm/sync/condvar.rs:148` to update the
> doc comment to say 'Wakes up the first thread of a process' instead of 'all threads.'"

### Classification

This bug was discovered through the verification **review process** — not by SMT solving, but
by the systematic comparison between the formal model and the source code that verification
methodology requires. The discipline of building an exact model forced careful reading of both
the code and its documentation.

---

## 5. Proposed Fix

### Option A: Fix the Documentation (if first-match is intentional)

```rust
/// # Description
///
/// Wakes up the first thread of a process that is waiting on the target condition variable.
```

### Option B: Fix the Implementation (if all-threads was intended)

```rust
pub unsafe fn notify_process(&self, pid: ProcessIdentifier) -> Result<(), Error> {
    loop {
        let idx: Option<usize> = self
            .inner.sleeping.borrow()
            .iter()
            .position(|&(p, _)| p == pid);
        match idx {
            Some(at) => {
                let (_, tid) = self.inner.sleeping.borrow_mut().remove(at);
                ProcessManager::wakeup(tid)?;
            },
            None => break,
        }
    }
    Ok(())
}
```

### Recommendation

Determine the **design intent** before fixing. If `notify_process` was meant to be a
per-process `notify_one`, Option A is correct. If it was meant to be a per-process
`notify_all`, Option B is needed — and requires verification model updates.

---

## 6. File Reference Index

| File | Role |
|------|------|
| `src/kernel/src/pm/sync/condvar.rs` | Original source (doc bug at line ~130) |
| `verus/split/kernel/pm/sync/condvar.rs` | Verus exec model (follows implementation) |
| `verus-ai-history/reviews/condvar/claude_r2_a1.md` | AI review identifying the doc bug |
| `verus-ai-history/reviews/condvar/exec-consistency_20260214_114445.md` | Consistency report confirming divergence |
