# Bug Report: Mutex/Condvar Capacity Check — Premature Rejection of Existing Entries

> **Bug ID:** #4 (from `bugs.md`)
> **Severity:** MEDIUM
> **Status:** Confirmed real bug, not yet fixed
> **Discovery Method:** Direct — identified by AI prover during Verus formal verification modeling
> **Affected Files:**
> [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs) (primary),
> [`src/kernel/src/pm/kcall/lock_mutex.rs`](src/kernel/src/pm/kcall/lock_mutex.rs),
> [`src/kernel/src/pm/kcall/wait_cond.rs`](src/kernel/src/pm/kcall/wait_cond.rs),
> [`src/kernel/src/pm/kcall/signal_cond.rs`](src/kernel/src/pm/kcall/signal_cond.rs)
> **Also Affects:** `get_cond()` at the same file — identical bug pattern

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Affected Call Chain](#3-affected-call-chain)
4. [Trigger Scenario and Impact](#4-trigger-scenario-and-impact)
5. [How Verus Verification Directly Discovered This Bug](#5-how-verus-verification-directly-discovered-this-bug)
6. [How the AI Prover Encoded the Bug with Explicit Annotation](#6-how-the-ai-prover-encoded-the-bug-with-explicit-annotation)
7. [The AI Review Process](#7-the-ai-review-process)
8. [The Verification Model's Postcondition Analysis](#8-the-verification-models-postcondition-analysis)
9. [Proposed Fix](#9-proposed-fix)
10. [Comparison with the Quantum Inheritance Bug](#10-comparison-with-the-quantum-inheritance-bug)
11. [File Reference Index](#11-file-reference-index)

---

## 1. Executive Summary

The `get_mutex()` and `get_cond()` functions in Nanvix's process state module perform a capacity
check **before** consulting whether the requested address already exists in the map. When the
`BTreeMap` is at its capacity limit (`MUTEX_OPEN_MAX = 32` / `COND_OPEN_MAX = 32`), the function
rejects **all** requests — including requests for mutexes/condvars that are **already present** in
the map and would not cause it to grow.

This is a logic error: the `or_insert_with()` call on an existing key is a no-op (no insertion),
so the capacity check should not apply. The result is that once a process holds exactly 32 mutexes,
it cannot even re-acquire a mutex it already owns — `lock_mutex()` syscall returns
`ErrorCode::OutOfMemory` spuriously.

Unlike the Quantum Inheritance bug (Bug #10) which was discovered **indirectly** through the
verification modeling process, this bug was discovered **directly** by the AI prover. While
building the Verus verification model for `ProcessState`, the prover had to precisely model the
`get_mutex()` semantics and explicitly noted the capacity-check ordering as a "Known
over-approximation (inherited from original)" — a polite way of saying "this is a bug in the
original code that we faithfully reproduce."

---

## 2. The Bug

### Root Cause

In [`src/kernel/src/pm/process/state/mod.rs` (lines 295–308)](src/kernel/src/pm/process/state/mod.rs#L295-L308),
the `get_mutex()` function checks capacity **before** consulting the entry:

```rust
// mod.rs:295-308
pub fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
    // Check if maximum number of mutexes has been reached.
    if self.mutexes.len() >= MUTEX_OPEN_MAX {           // ← Step 1: capacity check FIRST
        let reason: &'static str = "maximum number of mutexes reached";
        error!("{:?} (addr={:#x?})", reason, mutex_addr);
        return Err(Error::new(ErrorCode::OutOfMemory, reason));
    }

    Ok(self
        .mutexes
        .entry(mutex_addr)                               // ← Step 2: lookup/insert SECOND
        .or_insert_with(Mutex::new)                       // (no-op if key already exists)
        .clone())
}
```

The function serves **dual purposes** (as documented in its own docstring at
[lines 284–293](src/kernel/src/pm/process/state/mod.rs#L284-L293)):

> *"On success, the mutex that is associated with the given address is returned.
> If no mutex is associated with the given address, a new mutex is created and returned."*

In other words:
1. **Retrieve** an existing mutex (key already in `BTreeMap` → `entry().or_insert_with()` is a no-op).
2. **Create** a new mutex (key not in `BTreeMap` → inserts a new entry, growing the map).

The capacity check at line 297 does not distinguish between these two cases. It rejects **both**
when `self.mutexes.len() >= 32`.

### The `get_cond()` Twin

The exact same bug exists in `get_cond()` at
[lines 354–367](src/kernel/src/pm/process/state/mod.rs#L354-L367):

```rust
// mod.rs:354-367
pub fn get_cond(&mut self, cond_addr: ConditionAddress) -> Result<Condvar, Error> {
    if self.conditions.len() >= COND_OPEN_MAX {          // ← Same premature check
        let reason: &'static str = "maximum number of condition variables reached";
        error!("{:?} (addr={:#x?})", reason, cond_addr);
        return Err(Error::new(ErrorCode::OutOfMemory, reason));
    }

    Ok(self
        .conditions
        .entry(cond_addr)
        .or_insert_with(Condvar::new)
        .clone())
}
```

### Configuration

The capacity limits are defined in
[`build/kernel_config.toml` (lines 37–45)](build/kernel_config.toml#L37-L45):

```toml
# Maximum number of mutexes a process may hold at the same time.
# Notes:
# - This number was arbitrarily chosen.
mutex_open_max = 32

# Maximum number of condition variables a process may hold at the same time.
# Notes:
# - This number was arbitrarily chosen.
cond_open_max = 32
```

The limit of 32 is intentional — a microkernel needs to bound per-process resource usage. The
bug is not in the limit itself, but in **when** the limit is enforced.

---

## 3. Affected Call Chain

The bug affects all syscall paths that acquire mutexes or condition variables:

### `lock_mutex` Syscall

[`src/kernel/src/pm/kcall/lock_mutex.rs` (lines 63–94)](src/kernel/src/pm/kcall/lock_mutex.rs#L63-L94):

```rust
pub unsafe fn lock_mutex(...) -> Result<(), SleepError> {
    // ...
    let mutex: Mutex = ProcessManager::get_mutex(mutex_addr)  // ← Can fail here!
        .map_err(SleepError::Generic)?;
    let guard: MutexGuard = mutex.lock(timeout)?;
    ProcessManager::put_mutex_guard(mutex_addr, guard)
        .map_err(SleepError::Generic)
}
```

`ProcessManager::get_mutex()` at
[`mod.rs` line 1256](src/kernel/src/pm/process/manager/mod.rs#L1256) delegates directly:

```rust
fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
    self.get_running_mut().state_mut().get_mutex(mutex_addr)  // ← Calls buggy function
}
```

### `wait_cond` Syscall

[`src/kernel/src/pm/kcall/wait_cond.rs` (lines 117, 133)](src/kernel/src/pm/kcall/wait_cond.rs#L117):

```rust
match ProcessManager::get_cond(cond_addr) {  // ← Can fail at capacity
    // ...
}
let mutex: Mutex = ProcessManager::get_mutex(mutex_addr)  // ← Can also fail
    .map_err(SleepError::Generic)?;
```

`wait_cond` calls **both** `get_cond()` and `get_mutex()` — either can be affected.

### `signal_cond` Syscall

[`src/kernel/src/pm/kcall/signal_cond.rs` line 64](src/kernel/src/pm/kcall/signal_cond.rs#L64):

```rust
let cond: Condvar = ProcessManager::get_cond(cond_addr)?;  // ← Can fail at capacity
```

### Mutex Lifecycle

The complete mutex lifecycle is:

```
lock_mutex:
    get_mutex(addr)    → retrieves/creates Mutex (Arc clone), stores in BTreeMap
    mutex.lock()       → acquires the lock
    put_mutex_guard()  → stores MutexGuard in thread state

unlock_mutex:
    take_mutex_guard() → removes MutexGuard from thread state (drops the lock)
    put_mutex(addr)    → removes BTreeMap entry IF reference_count() <= 2
```

The `put_mutex()` at [`mod.rs` lines 323–337](src/kernel/src/pm/process/state/mod.rs#L323-L337)
uses `extract_if` with the predicate `mutex.reference_count() <= 2` to only remove a mutex
when no other thread holds a reference to it. This means the `BTreeMap` can remain full even
while mutexes are actively being locked and unlocked.

---

## 4. Trigger Scenario and Impact

### Trigger Conditions

1. A process creates and locks 32 different mutexes (reaching `MUTEX_OPEN_MAX`).
2. The process attempts to lock **any** of those 32 mutexes again (e.g., from a different thread).
3. `get_mutex()` fails with `ErrorCode::OutOfMemory` even though the mutex already exists.

### Concrete Example

```
Thread A locks mutex_1, mutex_2, ..., mutex_32      → All succeed (BTreeMap grows to 32)
Thread B tries to lock mutex_1                       → FAILS with OutOfMemory!
```

Thread B's `lock_mutex(mutex_1)` calls `get_mutex(mutex_1)`. Even though `mutex_1` is already
in the `BTreeMap` and `entry(mutex_1).or_insert_with(Mutex::new)` would simply return the existing
entry without growing the map, the capacity check `self.mutexes.len() >= 32` triggers first and
returns an error.

### Impact Assessment

**Not a permanent denial of service.** The error is transient:
- When any thread unlocks a mutex **and** that mutex's `Arc::reference_count()` drops to ≤ 2,
  `put_mutex()` removes the entry from the `BTreeMap`, reducing `len()` below 32.
- Subsequent `get_mutex()` calls succeed again.

**But the window is real:**
- While the `BTreeMap` contains 32 entries, **all** `lock_mutex` and `wait_cond` calls that
  go through `get_mutex()` / `get_cond()` will fail — even for addresses already in the map.
- The error propagates as `SleepError::Generic(Error { code: OutOfMemory })` to userspace.
- A well-behaved application that expects `lock_mutex` to succeed on a mutex it already created
  will encounter unexpected failures.

**Practical trigger difficulty:** A process needs to hold 32 **distinct** mutex addresses
simultaneously. This is uncommon in simple applications but entirely possible in complex
concurrent software with fine-grained locking.

---

## 5. How Verus Verification Directly Discovered This Bug

Unlike the Quantum Inheritance bug (Bug #10), which was discovered **indirectly** through the
verification modeling process, the mutex capacity bug was identified **directly** by the AI prover
during its attempt to build a faithful Verus model.

### The Discovery Moment

The critical log entry is in the prover fix session at
[`verus-ai-history/logs/process_state/prover_fix_claude_20260209_201712.txt`](verus-ai-history/logs/process_state/prover_fix_claude_20260209_201712.txt):

> *"The original checks `self.mutexes.len() >= MUTEX_OPEN_MAX` first. If `mutex_addr` already
> exists, `len()` hasn't changed, so `len()` would be exactly `MUTEX_OPEN_MAX` (at capacity).
> The check fails even though `or_insert_with` wouldn't add a new entry. This IS a latent bug
> in the original — the check should only reject when `!already_present`. Our model faithfully
> replicates this behavior. I'll document it."*

The AI prover explicitly called it a **"latent bug in the original"** — not a design choice, not
an acceptable trade-off, but a bug. However, rather than introducing a corrected specification
(which would diverge from the original code), the prover chose to faithfully replicate the buggy
behavior and annotate it prominently.

### Why Verification Forced This Discovery

To build the Verus model, the prover had to decompose `get_mutex()` into its semantic components:

1. **Capacity check:** Is the map full?
2. **Presence check:** Is the key already in the map?
3. **Action:** Insert new entry OR return existing entry (clone).

In the original Rust code, steps 2 and 3 are merged in a single `entry().or_insert_with().clone()`
call, which handles both cases implicitly. But in the Verus model, the prover had to make
these cases **explicit** — the model takes an `already_present: bool` parameter that the caller
must provide:

```rust
pub fn get_mutex(
    &mut self,
    mutex_addr: Ghost<int>,
    already_present: bool,        // ← Caller must state whether key exists
) -> (result: Result<Ghost<nat>, Error>)
```

This decomposition immediately revealed the logical inconsistency: the capacity check at the top
applies to **both** the `already_present == true` and `already_present == false` branches, but
only the `already_present == false` branch actually grows the map.

---

## 6. How the AI Prover Encoded the Bug with Explicit Annotation

### The "Known Over-Approximation" Annotation

The AI prover added an explicit documentation block to the Verus model at
[`verus/split/kernel/pm/process/state/process_state.rs` (lines 296–299)](verus/split/kernel/pm/process/state/process_state.rs#L296-L299):

```rust
/// **Known over-approximation (inherited from original):** The capacity check
/// `self.mutexes.len() >= MUTEX_OPEN_MAX` runs before consulting the entry.
/// At capacity, this rejects even existing keys whose `or_insert_with` would
/// not grow the map. The spec faithfully models this behavior.
```

The same annotation was added to `get_cond()` at
[lines 458–459](verus/split/kernel/pm/process/state/process_state.rs#L458-L459):

```rust
/// **Known over-approximation (inherited from original):** Same as `get_mutex` —
/// capacity check runs before consulting the entry, rejecting existing keys at capacity.
```

The term "over-approximation" is a formal verification term meaning "the model permits more
error behaviors than necessary." In this case, the model (and the original code) can return
`OutOfMemory` even when no memory allocation is needed — an over-approximation of the failure
condition.

### The Module Header Documentation

The module header at
[`process_state.rs` (lines 15–17)](verus/split/kernel/pm/process/state/process_state.rs#L15-L17)
summarizes:

```rust
//! - `get_mutex` enforces capacity bound (MUTEX_MAX = 32): returns `OutOfMemory`
//!   error **only** when the map is at capacity (spec_mutexes_full), inserts new
//!   [entries] when not at capacity.
```

Note the careful phrasing: it says the error occurs "when the map is at capacity" — it does
**not** say "when the map is at capacity **and** the key is new." This is the faithful encoding
of the buggy behavior.

---

## 7. The AI Review Process

### Round 1, Attempt 1: Claude Identifies the Root Cause

In [`verus-ai-history/reviews/process_state/claude_r1_a1.md`](verus-ai-history/reviews/process_state/claude_r1_a1.md),
the first review did not yet focus on the capacity check — it was concerned with larger issues
like missing reference-count modeling and incorrect `MUTEX_MAX` constants (256 instead of 32).

### Round 1, Attempt 2: Claude Flags the Bug Explicitly

In [`verus-ai-history/reviews/process_state/claude_r1_a2.md`](verus-ai-history/reviews/process_state/claude_r1_a2.md),
after the prover added reference-count modeling, Claude identified the capacity check issue:

> *"Both the original and verified code return `OutOfMemory` when the map is at capacity, even
> if the requested address is already present (capacity check precedes the presence check). This
> means an existing mutex cannot be retrieved when the map is full. The verified code faithfully
> reproduces this behavior, which may be an overly conservative check in the original. This is
> not a verification defect — it's a faithful modeling of potentially suboptimal original logic."*

This review was cautious — it called it "potentially suboptimal" rather than a definitive bug.

### Round 2, Attempt 1: Claude Escalates to High

In [`verus-ai-history/reviews/process_state/claude_r2_a1.md`](verus-ai-history/reviews/process_state/claude_r2_a1.md),
the reviewer became more specific:

> *"The spec's error postcondition says only `old(self).spec_mutexes_full()`, but does not assert
> `!already_present`. This means the spec permits an error even when the key exists — matching
> the original's overly-conservative check — but a tighter spec that allows success when
> `already_present` regardless of capacity would be more useful for callers and would catch this
> latent design bug."*
>
> **Suggested Fix:** *"Add the postcondition `result is Err ==> !already_present`"*

This suggestion is significant: it proposes a postcondition that the **original code cannot
satisfy** — thereby formally proving the original code is buggy. If the Verus model were changed
to require `result is Err ==> !already_present`, the implementation would fail to verify,
because the capacity check can return `Err` even when `already_present == true`.

### Round 1, Attempt 3: Acknowledged but Not Fixed

In [`verus-ai-history/reviews/process_state/claude_r1_a3.md`](verus-ai-history/reviews/process_state/claude_r1_a3.md),
the review acknowledged the issue was correctly documented:

> *"N/A. Correctly identified in R2 as faithful modeling of original behavior (the original also
> rejects existing-key lookups when at capacity). No fix needed or attempted."*

### GPT's Independent Assessment

In [`verus-ai-history/reviews/process_state/gpt_r1_a1.md`](verus-ai-history/reviews/process_state/gpt_r1_a1.md),
GPT independently noted the issue from a different angle — the `already_present` oracle parameter:

> *"Verified functions such as `get_mutex`, `put_mutex`, `get_cond`, `put_cond` [...] require
> caller-supplied booleans (`already_present`, `contains`, `ref_count_at_threshold`) without any
> verified wrapper that computes them from actual data structures. This means the verification
> does not establish semantic equivalence with the original implementation."*

GPT identified that the model's structure (separate `already_present` parameter) inherently
exposes the logical gap in the original code — the capacity check ignores the `already_present`
information.

---

## 8. The Verification Model's Postcondition Analysis

### The Postconditions That Encode the Bug

The `get_mutex()` model at
[`process_state.rs` (lines 317–349)](verus/split/kernel/pm/process/state/process_state.rs#L317-L349)
has two postcondition branches:

**Success path (`result is Ok`):**
```rust
result is Ok ==> {
    &&& self.spec_has_mutex(mutex_addr@)
    // If was present: ref count incremented by 1 (modeling clone()).
    &&& already_present ==>
            self.spec_mutex_ref_count(mutex_addr@)
                == old(self).spec_mutex_ref_count(mutex_addr@) + 1
    // If was absent: new entry with ref_count = 2.
    &&& !already_present ==> self.spec_mutex_ref_count(mutex_addr@) == 2
    // ... frame conditions ...
    &&& self.wf()
},
```

**Error path (`result is Err`) — THE BUG:**
```rust
result is Err ==> {
    &&& result->Err_0.code == ErrorCode::OutOfMemory
    &&& old(self).spec_mutexes_full()     // ← Only requires "map is full"
    // NOTE: Does NOT require !already_present
    // This means error is possible even when already_present == true
    &&& self.spec_mutex_count() == old(self).spec_mutex_count()  // Map unchanged
    // ... frame conditions ...
    &&& self.wf()
},
```

The error postcondition says: "if the result is an error, then the map was full." It does **not**
say "the map was full **and** the key was absent." This faithfully reflects the original code's
behavior — at capacity, even existing keys are rejected.

### What a Correct Postcondition Would Look Like

A corrected specification would add the constraint that errors only occur for genuinely new
entries:

```rust
// CORRECT postcondition (what the code SHOULD satisfy):
result is Err ==> {
    &&& result->Err_0.code == ErrorCode::OutOfMemory
    &&& old(self).spec_mutexes_full()
    &&& !already_present               // ← Only error when key is NEW
    // ...
},
```

If this corrected postcondition were applied to the **current** implementation, **Verus would
fail to verify it** — because the implementation rejects all requests at capacity, including
existing keys. This constitutes a formal proof that the original code is buggy with respect to
the intended specification.

### The `wf()` Invariant

The spec file at
[`process_state.spec.rs` (lines 119–147)](verus/split/kernel/pm/process/state/process_state.spec.rs#L119-L147)
defines `wf()` including the capacity bound:

```rust
pub open spec fn wf(&self) -> bool {
    &&& self.ghost_mutexes@.dom().finite()
    &&& self.ghost_mutexes@.dom().len() == self.mutex_count as nat
    // ...
    &&& self.mutex_count as nat <= Self::MUTEX_MAX() as nat    // ← capacity ≤ 32
    &&& self.cond_count as nat <= Self::COND_MAX() as nat
    // ...
}
```

And the "full" predicate at
[`process_state.spec.rs` (lines 149–151)](verus/split/kernel/pm/process/state/process_state.spec.rs#L149-L151):

```rust
pub open spec fn spec_mutexes_full(&self) -> bool {
    self.mutex_count as nat >= Self::MUTEX_MAX() as nat    // count >= 32
}
```

Verus proves that `wf()` is preserved by `get_mutex()` in both the success and error paths.
The invariant `mutex_count <= 32` is maintained — the bug is not about the count exceeding the
limit, but about the limit being checked at the wrong point in the logic.

---

## 9. Proposed Fix

### Fix for `get_mutex()`

At [`src/kernel/src/pm/process/state/mod.rs` (lines 295–308)](src/kernel/src/pm/process/state/mod.rs#L295-L308):

```rust
// BEFORE (buggy):
pub fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
    if self.mutexes.len() >= MUTEX_OPEN_MAX {
        let reason: &'static str = "maximum number of mutexes reached";
        error!("{:?} (addr={:#x?})", reason, mutex_addr);
        return Err(Error::new(ErrorCode::OutOfMemory, reason));
    }

    Ok(self.mutexes.entry(mutex_addr).or_insert_with(Mutex::new).clone())
}

// AFTER (correct):
pub fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
    if !self.mutexes.contains_key(&mutex_addr) && self.mutexes.len() >= MUTEX_OPEN_MAX {
        let reason: &'static str = "maximum number of mutexes reached";
        error!("{:?} (addr={:#x?})", reason, mutex_addr);
        return Err(Error::new(ErrorCode::OutOfMemory, reason));
    }

    Ok(self.mutexes.entry(mutex_addr).or_insert_with(Mutex::new).clone())
}
```

The change adds `!self.mutexes.contains_key(&mutex_addr) &&` before the capacity check. This
ensures the error is only returned when:
1. The requested key does **not** already exist in the map, **AND**
2. The map is at capacity.

If the key already exists, the capacity check is skipped and the existing entry is returned
regardless of map size.

### Fix for `get_cond()`

At [`src/kernel/src/pm/process/state/mod.rs` (lines 354–367)](src/kernel/src/pm/process/state/mod.rs#L354-L367):

```rust
// BEFORE (buggy):
if self.conditions.len() >= COND_OPEN_MAX {

// AFTER (correct):
if !self.conditions.contains_key(&cond_addr) && self.conditions.len() >= COND_OPEN_MAX {
```

### Impact on Verification Model

The corrected `get_mutex()` Verus model would:

1. **Move the capacity check inside the `!already_present` branch:**
   ```rust
   if already_present {
       // Always succeeds — no capacity check needed.
       // Increment ref count and return.
   } else {
       if self.mutex_count >= Self::MUTEX_MAX_EXEC() {
           return Err(Error::new(ErrorCode::OutOfMemory, reason));
       }
       // Insert new entry.
   }
   ```

2. **Strengthen the error postcondition:**
   ```rust
   result is Err ==> {
       &&& result->Err_0.code == ErrorCode::OutOfMemory
       &&& old(self).spec_mutexes_full()
       &&& !already_present    // ← NEW: error only for genuinely new entries
       // ...
   },
   ```

3. **Strengthen the success postcondition:**
   ```rust
   result is Ok ==> {
       // ... existing conditions ...
       // NEW: success is guaranteed when the key already exists
       &&& already_present ==> result is Ok    // (always true, but makes it explicit)
   },
   ```

4. **Remove the "Known over-approximation" annotation** — it would no longer apply.

---

## 10. Comparison with the Quantum Inheritance Bug

| Aspect | Bug #4 (Mutex Capacity) | Bug #10 (Quantum Inheritance) |
|--------|------------------------|-------------------------------|
| **Discovery method** | Direct — AI prover identified it while building the model | Indirect — surfaced through modeling process |
| **Verus's role** | Forced decomposition of `entry().or_insert_with()` into explicit presence check, making the bug obvious | Forced explicit postconditions for `switch()` branches, making the quantum non-reset visible |
| **AI prover's response** | Labeled it "latent bug in the original" and documented as "Known over-approximation" | Labeled it "intentional" with a "Design note" rationalizing it as per-process quantum |
| **Honesty of encoding** | Honest — clearly identified as a bug inherited from the original | Misleading — rationalized as a design choice |
| **Specification fidelity** | Specification faithfully encodes buggy behavior; a corrected spec would fail verification | Specification faithfully encodes buggy behavior; fairness is out of scope |
| **Trigger difficulty** | Moderate — requires 32 distinct mutexes in one process | Easy — any multi-threaded single-process scenario |
| **Impact** | Transient — errors clear when mutexes are released | Permanent — 1000:1 thread starvation ratio persists |
| **Fix complexity** | One condition addition per function | One line moved |

### Key Insight

Both bugs demonstrate the same fundamental pattern in formal verification: **the specification
encoded the buggy behavior, so Verus confirmed the code is "correct" with respect to a wrong
spec.** The difference is in how the AI prover handled the discovery:

- For Bug #4, the prover **acknowledged** the bug and documented it explicitly.
- For Bug #10, the prover **rationalized** the bug as an intentional design choice.

This highlights the importance of **independent review** in AI-assisted verification: a prover
building a model may have incentives to make the model match the code (even if the code is wrong),
while a reviewer can challenge whether the code's behavior is actually intended.

---

## 11. File Reference Index

### Original Source Code

| File | Lines | Description |
|------|-------|-------------|
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L135) | 135 | `ProcessState` struct definition |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L151) | 151 | `mutexes: BTreeMap<MutexAddress, Mutex>` field |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L284-L293) | 284–293 | `get_mutex()` doc comment (states dual purpose) |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L295-L308) | 295–308 | **`get_mutex()` — Bug location** |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L297) | 297 | **The premature capacity check** |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L323-L337) | 323–337 | `put_mutex()` — mutex release with `extract_if` |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L333) | 333 | `extract_if` with `reference_count() <= 2` |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L354-L367) | 354–367 | **`get_cond()` — Same bug** |
| [`src/kernel/src/pm/process/state/mod.rs`](src/kernel/src/pm/process/state/mod.rs#L382-L396) | 382–396 | `put_cond()` — condvar release with `extract_if` |
| [`src/kernel/src/pm/kcall/lock_mutex.rs`](src/kernel/src/pm/kcall/lock_mutex.rs#L63-L94) | 63–94 | `lock_mutex` syscall — calls `get_mutex()` at line 92 |
| [`src/kernel/src/pm/kcall/wait_cond.rs`](src/kernel/src/pm/kcall/wait_cond.rs#L117) | 117 | `wait_cond` syscall — calls `get_cond()` |
| [`src/kernel/src/pm/kcall/wait_cond.rs`](src/kernel/src/pm/kcall/wait_cond.rs#L133) | 133 | `wait_cond` syscall — also calls `get_mutex()` |
| [`src/kernel/src/pm/kcall/signal_cond.rs`](src/kernel/src/pm/kcall/signal_cond.rs#L64) | 64 | `signal_cond` syscall — calls `get_cond()` |
| [`src/kernel/src/pm/process/manager/mod.rs`](src/kernel/src/pm/process/manager/mod.rs#L1256-L1258) | 1256–1258 | `ProcessManager::get_mutex()` — delegates to `ProcessState` |
| [`src/kernel/src/pm/process/manager/mod.rs`](src/kernel/src/pm/process/manager/mod.rs#L1275-L1277) | 1275–1277 | `ProcessManager::get_cond()` — delegates to `ProcessState` |
| [`build/kernel_config.toml`](build/kernel_config.toml#L37-L45) | 37–45 | `mutex_open_max = 32`, `cond_open_max = 32` |

### Verus Verification Model

| File | Lines | Description |
|------|-------|-------------|
| [`verus/split/kernel/pm/process/state/process_state.rs`](verus/split/kernel/pm/process/state/process_state.rs#L296-L299) | 296–299 | **"Known over-approximation" annotation** |
| [`verus/split/kernel/pm/process/state/process_state.rs`](verus/split/kernel/pm/process/state/process_state.rs#L314-L370) | 314–370 | `get_mutex()` Verus model with postconditions |
| [`verus/split/kernel/pm/process/state/process_state.rs`](verus/split/kernel/pm/process/state/process_state.rs#L340) | 340 | Error postcondition: `old(self).spec_mutexes_full()` (missing `!already_present`) |
| [`verus/split/kernel/pm/process/state/process_state.rs`](verus/split/kernel/pm/process/state/process_state.rs#L458-L459) | 458–459 | `get_cond()` over-approximation annotation |
| [`verus/split/kernel/pm/process/state/process_state.rs`](verus/split/kernel/pm/process/state/process_state.rs#L473-L530) | 473–530 | `get_cond()` Verus model with postconditions |
| [`verus/split/kernel/pm/process/state/process_state.spec.rs`](verus/split/kernel/pm/process/state/process_state.spec.rs#L119-L147) | 119–147 | `wf()` predicate with capacity bounds |
| [`verus/split/kernel/pm/process/state/process_state.spec.rs`](verus/split/kernel/pm/process/state/process_state.spec.rs#L149-L151) | 149–151 | `spec_mutexes_full()` predicate |
| [`verus/split/kernel/pm/process/state/process_state.spec.rs`](verus/split/kernel/pm/process/state/process_state.spec.rs#L160-L162) | 160–162 | `MUTEX_MAX()` spec constant = 32 |

### AI Review History

| File | Role | Key Finding |
|------|------|-------------|
| [`verus-ai-history/logs/process_state/prover_fix_claude_20260209_201712.txt`](verus-ai-history/logs/process_state/prover_fix_claude_20260209_201712.txt) | AI Prover | **First identification**: "This IS a latent bug in the original" |
| [`verus-ai-history/reviews/process_state/claude_r1_a1.md`](verus-ai-history/reviews/process_state/claude_r1_a1.md) | Claude R1 A1 | Initial review — focused on ref-count and constant issues |
| [`verus-ai-history/reviews/process_state/claude_r1_a2.md`](verus-ai-history/reviews/process_state/claude_r1_a2.md) | Claude R1 A2 | First reviewer mention: "potentially suboptimal original logic" |
| [`verus-ai-history/reviews/process_state/claude_r2_a1.md`](verus-ai-history/reviews/process_state/claude_r2_a1.md) | Claude R2 A1 | Escalation: suggests `result is Err ==> !already_present` |
| [`verus-ai-history/reviews/process_state/claude_r1_a3.md`](verus-ai-history/reviews/process_state/claude_r1_a3.md) | Claude R1 A3 | Acknowledged as faithful modeling, no fix attempted |
| [`verus-ai-history/reviews/process_state/gpt_r1_a1.md`](verus-ai-history/reviews/process_state/gpt_r1_a1.md) | GPT R1 A1 | Noted oracle parameter gap exposes logical inconsistency |

### Bug Documentation

| File | Description |
|------|-------------|
| [`bugs.md`](bugs.md) | Full bug catalog — Bug #4 with impact analysis |

---

*Report generated on 2026-02-10. Based on commit `fcbdf39` on branch `integration`.*
