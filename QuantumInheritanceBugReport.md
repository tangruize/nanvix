# Bug Report: Quantum Inheritance — Permanent Thread Starvation

> **Bug ID:** #10 (from `bugs.md`)
> **Severity:** MEDIUM (upgraded from LOW after deep analysis)
> **Status:** Confirmed real bug, not yet fixed
> **Discovery Method:** Indirect — surfaced during AI-assisted Verus formal verification modeling
> **Affected File:** [`src/kernel/src/pm/process/manager/unsafe.rs`](src/kernel/src/pm/process/manager/unsafe.rs)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Bug](#2-the-bug)
3. [Trigger Scenario and Impact](#3-trigger-scenario-and-impact)
4. [How Verus Verification Helped Discover This Bug](#4-how-verus-verification-helped-discover-this-bug)
5. [How the AI Prover Wrongly Encoded the Bug as Intended Behavior](#5-how-the-ai-prover-wrongly-encoded-the-bug-as-intended-behavior)
6. [The Review Process That Challenged the Encoding](#6-the-review-process-that-challenged-the-encoding)
7. [Why Verus Did Not Directly Catch the Bug](#7-why-verus-did-not-directly-catch-the-bug)
8. [Proposed Fix](#8-proposed-fix)
9. [Lessons Learned](#9-lessons-learned)
10. [File Reference Index](#10-file-reference-index)

---

## 1. Executive Summary

A scheduling fairness bug exists in the Nanvix kernel's context switch logic. When threads within
the **same process** are switched, the time quantum (remaining CPU ticks) is **not reset**, causing
the new thread to inherit the previous thread's nearly-exhausted quantum. This leads to a stable,
permanent **1000:1 thread starvation ratio** (equal to `SCHEDULER_FREQ`).

The bug was not found by Verus's SMT solver outputting "verification failed." Instead, it was
discovered through the **process** of building a Verus verification model: the AI prover was forced
to write precise per-branch postconditions for the `switch()` function, which made the
quantum-inheritance behavior **explicitly visible** in formal notation. A subsequent AI reviewer
(Claude) challenged the prover's "Design note" that rationalized this as intentional behavior,
and a full timing analysis confirmed it as a genuine bug.

---

## 2. The Bug

### Root Cause

In [`src/kernel/src/pm/process/manager/unsafe.rs` (lines 770–778)](src/kernel/src/pm/process/manager/unsafe.rs#L770-L778),
the `switch()` function only resets `REMAINING_QUANTUM` when the **process ID changes**:

```rust
// unsafe.rs:770-778
if next_tid != previous_tid {
    // Hard context switch.
    if next_pid != previous_pid {
        // Quantum is ONLY reset here — when PID changes.
        REMAINING_QUANTUM.store(SCHEDULER_FREQ, ORDER);  // Line 775
        CURRENT_PID.store(next_pid.into(), ORDER);        // Line 776
    }
    CURRENT_TID.store(next_tid.into(), ORDER);
}
```

When a **thread switch occurs within the same process** (same PID, different TID), the quantum is
**not reset**. The new thread inherits whatever quantum value the previous thread left behind.

### The Quantum State Machine

`REMAINING_QUANTUM` is a global `AtomicUsize` declared at
[`unsafe.rs` line 99](src/kernel/src/pm/process/manager/unsafe.rs#L99):

```rust
static REMAINING_QUANTUM: AtomicUsize = AtomicUsize::new(SCHEDULER_FREQ);
```

There are **exactly 2 write sites** in the entire codebase:

| Location | Operation | Condition |
|----------|-----------|-----------|
| [`unsafe.rs:499`](src/kernel/src/pm/process/manager/unsafe.rs#L499) | `store(remaining_ticks - 1)` | Every timer tick when quantum > 1 |
| [`unsafe.rs:775`](src/kernel/src/pm/process/manager/unsafe.rs#L775) | `store(SCHEDULER_FREQ)` | **Only** when `next_pid != previous_pid` |

No other code path can reset the quantum. This means that when `giveup()` triggers a context
switch to a thread in the **same process**, the quantum stays at 1.

### The `giveup()` Trigger

In [`unsafe.rs` lines 494–522](src/kernel/src/pm/process/manager/unsafe.rs#L494-L522), `giveup()`
is called on every timer interrupt:

```rust
// unsafe.rs:494-522
pub unsafe fn giveup() -> Result<(), Error> {
    let remaining_ticks: usize = REMAINING_QUANTUM.load(ORDER);
    if remaining_ticks > 1 {
        // Still has quantum — decrement and return.
        REMAINING_QUANTUM.store(remaining_ticks - 1, ORDER);
    } else {
        // Quantum exhausted (remaining_ticks <= 1) — context switch!
        let (next_pid, next_tid, from, to, user_tda) =
            Self::get_mut().try_borrow_mut()?.schedule();
        Self::switch(next_pid, next_tid, from, to, user_tda);
    }
    Ok(())
}
```

The condition `remaining_ticks <= 1` triggers a context switch. After `switch()` returns (to the
newly scheduled thread), if the quantum was **not reset** (same-PID case), it remains at 1. On the
**very next timer tick**, `giveup()` sees `remaining_ticks == 1 <= 1` and immediately triggers
**another** context switch.

---

## 3. Trigger Scenario and Impact

### Trigger Conditions

1. A single user process `P` (PID=1) has multiple threads (A, B, C).
2. The kernel process `K` (PID=0) is the only other process.
3. Thread A completes a full quantum (1000 ticks).

### Precise Timing Trace (3-Thread Scenario)

Assume `SCHEDULER_FREQ = 1000`. The scheduler uses
[`take_earliest_ready()`](src/kernel/src/pm/process/manager/mod.rs#L1352-L1372) which selects
the process with the **earliest admission time** (FIFO ordering).

| Time | Event | `REMAINING_QUANTUM` | Selected | Reason |
|------|-------|--------------------:|----------|--------|
| T=0 | All threads admitted (admission=0) | 1000 | K boots | — |
| T=1000 | K exhausts quantum → `schedule()` | → | P selected | P.earliest(0) < K(1000) |
| T=1000 | PID: 0→1 (change!) | **1000** | A runs | quantum reset ✓ |
| T=2000 | A exhausts quantum → `schedule()` | 1 | — | remaining ≤ 1 → switch |
| T=2000 | P.earliest = min(B=0, C=0, A=2000) = 0 | → | **P again!** | 0 < K(1000) |
| T=2000 | PID: 1→1 (**same!**) | **1 (NOT RESET!)** | B runs | B.admission=0 |
| T=2001 | remaining=1, ≤1 → **immediate switch!** | 1 | — | B got only **1 tick** |
| T=2001 | P.earliest = min(C=0, A=2000, B=2001) = 0 | → | **P again!** | 0 < K(1000) |
| T=2001 | Same PID | 1 | C runs | C.admission=0 |
| T=2002 | Immediate switch | 1 | — | C got only **1 tick** |
| T=2002 | P.earliest = min(A=2000, B=2001, C=2002) = 2000 | → | **K selected** | K(1000) < P(2000) |
| T=2002 | PID: 1→0 (change!) | **1000** | K runs | quantum finally reset |
| T=3002 | K exhausts quantum → P selected → PID: 0→1 | **1000** | A runs | A=2000 is earliest |
| T=4002 | **Cycle repeats identically** | | | |

### CPU Allocation Per Cycle

| Thread | Ticks per cycle | CPU share | Fair share |
|--------|----------------:|----------:|-----------:|
| A | 1000 | 49.95% | 16.7% |
| B | **1** | **0.05%** | 16.7% |
| C | **1** | **0.05%** | 16.7% |
| K | 1000 | 49.95% | 50% |

**Unfairness ratio: A : B = 1000 : 1** (equals `SCHEDULER_FREQ`).

### Why It Is Permanent

The relative `admission_time` ordering **never changes**. At the end of each cycle:
- A is pushed back with `admission = T`
- B is pushed back with `admission = T+1`
- C is pushed back with `admission = T+2`

A **always** has the earliest admission time and is **always** selected first when process P is
chosen. The 1-tick gap between B and C is structural. No built-in mechanism can break this cycle.

### Formal Liveness Violation

The system violates **bounded fairness**:

> ∀ ready threads t₁, t₂ in the same process, in any time window,
> CPU_time(t₁) / CPU_time(t₂) ≤ K for some bounded constant K.

Currently K = 1000. A fair scheduler should guarantee K ≈ 1.

---

## 4. How Verus Verification Helped Discover This Bug

The Verus formal verification process did **not** directly catch this bug through a failed proof.
Instead, it acted as a **catalyst** by forcing a level of precision that made the buggy behavior
visible to human and AI reviewers. Here is the detailed chain of events:

### Phase 1: Initial Verification Model (R1 A1)

The AI prover (Claude) created the first Verus verification model for `process_manager_unsafe`.
The initial `switch()` function accepted `new_inner: ProcessManagerInner` as a parameter,
conflating the inner state mutation with the atomic updates.

**Claude reviewer (R1 A1)** flagged this as a High issue:

> *"By accepting `new_inner` as a parameter, the verified model conflates the inner mutation with
> the context switch, making it impossible to verify that the inner state transition and the
> atomic update happen in the correct order and are consistent with each other."*
>
> — [`verus-ai-history/reviews/process_manager_unsafe/claude_r1_a1.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r1_a1.md)

### Phase 2: Separation Introduces a Verification Model Bug (R1 A2)

The AI prover responded by splitting `switch()` into two functions: `update_inner()` (inner state
mutation) and `switch()` (atomic updates). However, `update_inner()` prematurely set
`self.current_pid = next_pid` to maintain `wf()`. This caused `switch()` to see
`next_pid == self.current_pid` **always**, meaning the quantum reset branch **never fired** — not
even for cross-process switches.

**Claude reviewer (R1 A2)** caught this with a precise execution trace:

> *"Concrete trace for `exit()` with PID change (old_pid=1, next_pid=2):*
> - *Original: inner.exit() → CURRENT_PID still 1 → switch() sees 2≠1 → quantum reset ✓*
> - *Model: update_inner() → current_pid=2 → switch() sees 2==2 → NO quantum reset ✗"*
>
> — [`verus-ai-history/reviews/process_manager_unsafe/claude_r1_a2.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r1_a2.md)

**This was a bug in the verification model, not the original code.** But fixing it forced the
prover to deeply understand the quantum reset semantics.

### Phase 3: Fix Reveals the Original Code's Behavior (R1 A3)

The AI prover fixed the model bug by combining inner mutation and atomic updates back into a
single `switch()` function, correctly comparing `next_pid` against the **old** `self.current_pid`.
Claude reviewer (R1 A3) confirmed the fix:

> *"switch() line 277: self.inner = new_inner (current_pid still 1)*
> *switch() line 285: next_pid(2) != self.current_pid(1) → TRUE → quantum reset ✓"*
>
> — [`verus-ai-history/reviews/process_manager_unsafe/claude_r1_a3.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r1_a3.md)

But in this process, the prover had to write **explicit per-branch postconditions** for all three
cases of `switch()` — and this is where the bug became visible.

---

## 5. How the AI Prover Wrongly Encoded the Bug as Intended Behavior

In the final verification model at
[`verus/split/kernel/pm/process/manager/process_manager_unsafe.rs` (lines 346–365)](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L346-L365),
the `switch()` function's postconditions explicitly encode three branches:

```rust
ensures
    // ...
    // Branch 1: Hard switch WITH PID change → quantum reset ✓
    (next_tid != old(self).current_tid && next_pid != old(self).current_pid)
        ==> (self.current_pid == next_pid
             && self.remaining_quantum == self.scheduler_freq),

    // Branch 2: Hard switch, SAME PID → quantum UNCHANGED ← THE BUG
    (next_tid != old(self).current_tid && next_pid == old(self).current_pid)
        ==> (self.current_pid == old(self).current_pid
             && self.remaining_quantum == old(self).remaining_quantum),

    // Branch 3: Soft switch → quantum unchanged
    next_tid == old(self).current_tid
        ==> (self.current_pid == old(self).current_pid
             && self.remaining_quantum == old(self).remaining_quantum),
```

**Branch 2** is the critical one. It states: when a thread switch occurs within the same process,
`remaining_quantum == old(self).remaining_quantum` — the quantum is inherited unchanged. **Verus
successfully proved this postcondition**, meaning the SMT solver confirmed that the implementation
satisfies this specification.

The AI prover then added a **"Design note"** to rationalize this behavior (lines 354–358):

```rust
// Design note: same-PID hard switches (thread switch within same process)
// intentionally inherit the previous thread's remaining quantum. This matches
// the original code where REMAINING_QUANTUM is only reset on PID changes
// (unsafe.rs:775-777). The scheduler treats quantum as per-process, not
// per-thread: threads within the same process share the process's time slice.
```

This note makes two claims:
1. The behavior is **intentional** ("intentionally inherit").
2. The design rationale is **per-process quantum** ("scheduler treats quantum as per-process").

Both claims are wrong:
- The original code's comment at [line 519](src/kernel/src/pm/process/manager/unsafe.rs#L519)
  says *"updating the remaining quantum accordingly"* — indicating the **intent** was to properly
  update the quantum on every switch, not to leave it unchanged.
- There is no documentation anywhere in the codebase describing a "per-process quantum" design
  philosophy.

### The Spec File Also Encodes the Bug

The spec file at
[`verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs` (lines 134–145)](verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs#L134-L145)
defines `spec_quantum_valid`:

```rust
pub open spec fn spec_quantum_valid(&self) -> bool {
    self.remaining_quantum >= 1
    && self.remaining_quantum <= self.scheduler_freq
}
```

This is part of the well-formedness predicate `wf()` (line 162). The invariant states quantum
stays in `[1, scheduler_freq]` — which is technically maintained even with the bug. The quantum
never goes to 0 (decrement stops at 1, and the switch path either resets to `SCHEDULER_FREQ` or
leaves it unchanged). The spec correctly describes the **actual** behavior, but the actual behavior
is itself buggy.

### The `giveup()` Model Faithfully Reflects the Bug Too

The verification model's `giveup()` at
[`process_manager_unsafe.rs` line 443](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L443)
dispatches to `giveup_no_switch()` (line 393, quantum > 1 → decrement) or `giveup_with_switch()`
(line 414, quantum ≤ 1 → context switch via `switch()`). Since `switch()` may not reset quantum
(same-PID case), the next `giveup()` call will see `remaining_quantum == 1 ≤ 1` and immediately
trigger another switch — the starvation loop is faithfully modeled and proven correct by Verus.

---

## 6. The Review Process That Challenged the Encoding

The multi-model AI review process involved three AI reviewers (Claude, GPT, Gemini) across three
rounds (R1, R2, R3) with multiple attempts per round. The reviews are stored in
[`verus-ai-history/reviews/process_manager_unsafe/`](verus-ai-history/reviews/process_manager_unsafe/).

### Gemini (R1 A1): Praised the Quantum Management

> *"Unified Quantum Management: The `giveup()` model correctly unifies the 'decrement quantum'
> and 'context switch' paths, ensuring that the quantum invariant `1 <= remaining_quantum <=
> scheduler_freq` is always preserved."*
>
> — [`verus-ai-history/reviews/process_manager_unsafe/gemini_r1_a1.md`](verus-ai-history/reviews/process_manager_unsafe/gemini_r1_a1.md)

Gemini gave the module an **A** grade and did not notice the fairness issue.

### Claude (R3 A2): Accepted the Design Note

> *"switch() quantum not reset on same-PID hard switch — FIXED. The prover added a clear
> design note (exec lines 329–333) explaining that same-PID hard switches intentionally inherit
> quantum because the scheduler treats quantum as per-process, not per-thread."*
>
> — [`verus-ai-history/reviews/process_manager_unsafe/claude_r3_a2.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r3_a2.md)

Claude upgraded the module to an **A** grade, accepting the prover's rationalization.

### GPT (R1 A3): Flagged Fairness but at Wrong Level

GPT noted the verification doesn't prove starvation-freedom, but categorized it as an inherent
limitation of the abstract model rather than a symptom of an original code bug:

> *"The `switch` model carefully captures the stale-atomic PID comparison and quantum reset logic."*
>
> — [`verus-ai-history/reviews/process_manager_unsafe/gpt_r1_a3.md`](verus-ai-history/reviews/process_manager_unsafe/gpt_r1_a3.md)

### Post-Review Analysis: The Bug Surfaces

The bug was ultimately identified during the **post-review analysis phase** (documented in
[`bugs.md`](bugs.md)), where the complete timing trace was constructed. The key insight was
combining three facts:
1. `REMAINING_QUANTUM` is only reset on PID changes (from the verification model's Branch 2).
2. `giveup()` switches immediately when `remaining_quantum ≤ 1` (from the `giveup()` model).
3. `take_earliest_ready()` uses FIFO-like admission time ordering (from
   [`mod.rs:1352–1372`](src/kernel/src/pm/process/manager/mod.rs#L1352-L1372)).

None of these three facts alone reveals the bug. It is their **composition** — crossing the
boundary between the `unsafe.rs` module (quantum management) and the `mod.rs` module (scheduling
policy) — that produces the starvation.

---

## 7. Why Verus Did Not Directly Catch the Bug

Three fundamental reasons:

### 7.1 Fairness Was Explicitly Out of Scope

The verification model's spec file
([`process_manager_unsafe.spec.rs` lines 29–50](verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs#L29-L50))
documents that the ready queue is modeled as `Set<int>` (an unordered set). This abstraction
deliberately discards:
- `admission_time` ordering (needed to predict which thread is selected)
- The temporal accumulation of quantum across multiple switches
- Any notion of scheduling fairness

The `chosen_next` in `switch()` is an **oracle parameter** — an unconstrained value provided by
the caller. Verus proves that `switch()` is correct **for any** chosen_next, but cannot reason
about **which** thread will actually be chosen.

### 7.2 The Specification Encoded the Bug

Branch 2 of `switch()`'s postcondition states `remaining_quantum == old(self).remaining_quantum`
for same-PID hard switches. Verus proved this is satisfied by the implementation — because it **is**
what the implementation does. The specification faithfully reflects the buggy code, so Verus
confirms the code matches the spec. There is no "expected" spec to compare against.

### 7.3 The Bug Requires Cross-Module Composition

The starvation requires reasoning about:
1. `giveup()` in `unsafe.rs` (quantum exhaustion trigger)
2. `schedule()` in `mod.rs` (ready queue management)
3. `take_earliest_ready()` in `mod.rs` (FIFO-like selection)
4. `switch()` in `unsafe.rs` (quantum reset condition)
5. The temporal behavior across **multiple scheduling cycles**

The Verus model verifies each function in isolation (preserves `wf()`), but does not compose them
across time. A full fairness proof would require temporal logic (e.g., TLA+) or a ghost state
tracking per-thread CPU time across scheduling cycles.

---

## 8. Proposed Fix

A **one-line move** fixes the bug. Move the quantum reset **outside** the PID-change condition:

```rust
// BEFORE (unsafe.rs:770-778):
if next_tid != previous_tid {
    if next_pid != previous_pid {
        REMAINING_QUANTUM.store(SCHEDULER_FREQ, ORDER);  // Only on PID change
        CURRENT_PID.store(next_pid.into(), ORDER);
    }
    CURRENT_TID.store(next_tid.into(), ORDER);
}

// AFTER (proposed fix):
if next_tid != previous_tid {
    REMAINING_QUANTUM.store(SCHEDULER_FREQ, ORDER);  // Every thread switch
    if next_pid != previous_pid {
        CURRENT_PID.store(next_pid.into(), ORDER);
    }
    CURRENT_TID.store(next_tid.into(), ORDER);
}
```

This ensures every thread — regardless of whether it belongs to the same process — receives a
full quantum of `SCHEDULER_FREQ` ticks.

### Impact on Verification Model

The fix would change Branch 2 of `switch()`'s postcondition from:

```rust
// BEFORE: same-PID hard switch inherits quantum
(next_tid != old(self).current_tid && next_pid == old(self).current_pid)
    ==> self.remaining_quantum == old(self).remaining_quantum

// AFTER: same-PID hard switch also resets quantum
(next_tid != old(self).current_tid && next_pid == old(self).current_pid)
    ==> self.remaining_quantum == self.scheduler_freq
```

And the implementation body would change accordingly:

```rust
// BEFORE:
if next_tid != self.current_tid {
    if next_pid != self.current_pid {
        self.remaining_quantum = self.scheduler_freq;
        self.current_pid = next_pid;
    }
    self.current_tid = next_tid;
}

// AFTER:
if next_tid != self.current_tid {
    self.remaining_quantum = self.scheduler_freq;  // Always reset
    if next_pid != self.current_pid {
        self.current_pid = next_pid;
    }
    self.current_tid = next_tid;
}
```

---

## 9. Lessons Learned

### 9.1 Formal Verification Finds Bugs Through the Process, Not Just the Result

Verus's solver never said "verification failed" for this bug. The value came from the **modeling
discipline** — writing precise postconditions forced the buggy behavior into explicit, reviewable
notation. Without Verus, the same-PID quantum inheritance would have remained buried in an
`if`-nested control flow.

### 9.2 AI Provers Can Rationalize Bugs as Design Choices

The AI prover's "Design note" is a cautionary example. When asked to explain code behavior, an AI
will construct plausible-sounding rationales ("per-process quantum"). Multi-agent review (prover +
independent reviewers) helps, but in this case even the reviewers initially accepted the
rationalization. The bug was only caught when the **cross-module timing consequences** were fully
traced.

### 9.3 Specification-Level Bugs Are Invisible to Verification

If the specification faithfully encodes buggy behavior, the verifier will confirm correctness.
This is the classic "verifying the wrong thing" problem. Detecting it requires either:
- A higher-level specification (e.g., "all threads get fair CPU time")
- Cross-module composition that reveals emergent misbehavior
- External analysis (timing traces, simulation, testing)

### 9.4 Safety ≠ Liveness

Verus's `wf()` invariant proves **safety** (the system never enters a bad state). The quantum
inheritance bug is a **liveness** violation (a thread makes insufficient progress). Most current
verification tools focus on safety properties; liveness requires different techniques (temporal
logic, fairness assumptions, progress measures).

---

## 10. File Reference Index

### Original Source Code

| File | Lines | Description |
|------|-------|-------------|
| [`src/kernel/src/pm/process/manager/unsafe.rs`](src/kernel/src/pm/process/manager/unsafe.rs#L99) | 99 | `REMAINING_QUANTUM` declaration |
| [`src/kernel/src/pm/process/manager/unsafe.rs`](src/kernel/src/pm/process/manager/unsafe.rs#L494-L522) | 494–522 | `giveup()` — timer interrupt handler |
| [`src/kernel/src/pm/process/manager/unsafe.rs`](src/kernel/src/pm/process/manager/unsafe.rs#L758-L803) | 758–803 | `switch()` — context switch with quantum reset |
| [`src/kernel/src/pm/process/manager/unsafe.rs`](src/kernel/src/pm/process/manager/unsafe.rs#L770-L778) | 770–778 | **Bug location** — quantum only reset on PID change |
| [`src/kernel/src/pm/process/manager/mod.rs`](src/kernel/src/pm/process/manager/mod.rs#L640-L682) | 640–682 | `schedule()` — reschedule logic |
| [`src/kernel/src/pm/process/manager/mod.rs`](src/kernel/src/pm/process/manager/mod.rs#L1352-L1372) | 1352–1372 | `take_earliest_ready()` — FIFO-like process selection |

### Verus Verification Model

| File | Lines | Description |
|------|-------|-------------|
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L327-L390) | 327–390 | `switch()` verification model |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L346-L365) | 346–365 | **Postconditions encoding the bug** |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L354-L358) | 354–358 | AI prover's "Design note" rationalizing the bug |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L393-L412) | 393–412 | `giveup_no_switch()` — quantum decrement model |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.rs#L414-L441) | 414–441 | `giveup_with_switch()` — context switch model |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs#L134-L145) | 134–145 | `spec_quantum_valid()` — quantum range invariant |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs#L161-L170) | 161–170 | `wf()` — well-formedness predicate |
| [`verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs`](verus/split/kernel/pm/process/manager/process_manager_unsafe.spec.rs#L182-L184) | 182–184 | `spec_quantum_expired()` — expiration condition |

### AI Review History

| File | Role | Key Finding |
|------|------|-------------|
| [`verus-ai-history/reviews/process_manager_unsafe/claude_r1_a1.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r1_a1.md) | Claude R1 A1 | Flagged `switch()` conflating inner mutation with atomics |
| [`verus-ai-history/reviews/process_manager_unsafe/claude_r1_a2.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r1_a2.md) | Claude R1 A2 | Found verification model bug (`update_inner()` prematurely sets PID) |
| [`verus-ai-history/reviews/process_manager_unsafe/claude_r1_a3.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r1_a3.md) | Claude R1 A3 | Confirmed model fix; quantum semantics now explicit |
| [`verus-ai-history/reviews/process_manager_unsafe/gemini_r1_a1.md`](verus-ai-history/reviews/process_manager_unsafe/gemini_r1_a1.md) | Gemini R1 A1 | Praised quantum management (Grade A); missed fairness issue |
| [`verus-ai-history/reviews/process_manager_unsafe/claude_r3_a2.md`](verus-ai-history/reviews/process_manager_unsafe/claude_r3_a2.md) | Claude R3 A2 | Accepted "Design note" as intentional (Grade A) |
| [`verus-ai-history/reviews/process_manager_unsafe/gpt_r1_a3.md`](verus-ai-history/reviews/process_manager_unsafe/gpt_r1_a3.md) | GPT R1 A3 | Noted missing fairness verification but didn't flag original bug |

### Bug Documentation

| File | Description |
|------|-------------|
| [`bugs.md`](bugs.md) | Full bug catalog including Bug #10 with timing analysis |

---

*Report generated on 2026-02-10. Based on commit `fcbdf39` on branch `integration`.*
