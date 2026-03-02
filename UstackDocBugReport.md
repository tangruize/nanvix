# Bug Report: UserStack Documentation — Base/Top Address Semantics Reversed

> **Severity:** LOW
> **Status:** Confirmed; not yet fixed in production code
> **Discovery Method:** AI review — Verus consistency report identified that documentation
> contradicts the implementation
> **Affected File:** [`src/kernel/src/mm/ustack.rs`](src/kernel/src/mm/ustack.rs)
> **Verified File:** [`verus/split/kernel/mm/ustack.rs`](verus/split/kernel/mm/ustack.rs)
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

The `UserStack` struct's documentation states that `base` is "the highest address of the stack"
(since "stacks grow downwards"), implying that `top` should be the lowest address. However, the
implementation computes `top = base + size`, making `top` the **higher** address. The
documentation and implementation have opposite semantics for which end of the stack is "base"
and which is "top."

---

## 2. The Bug

### The Documentation

In [`src/kernel/src/mm/ustack.rs`](src/kernel/src/mm/ustack.rs):

```rust
/// # Notes
///
/// As sacks grow downwards, the base address is the highest address of the stack.
///            ^^^^^ (also typo: "sacks" → "stacks")
```

### The Implementation

```rust
pub fn top(&self) -> PageAligned<VirtualAddress> {
    PageAligned::from_raw_value(self.base.into_raw_value() + self.size()).unwrap()
}
```

### The Contradiction

| Term | Documentation Says | Implementation Does |
|------|-------------------|---------------------|
| `base` | Highest address (stack bottom) | Lowest address (allocation start) |
| `top`  | Implied: lowest address (stack top) | `base + size` = highest address |

If stacks grow downward:
- **Convention A (documentation):** `base` = high address (where stack starts growing down from)
- **Convention B (implementation):** `base` = low address (where the memory region starts),
  `top` = high address (where the memory region ends)

The implementation uses Convention B (memory region layout), while the documentation describes
Convention A (stack growth semantics). These are contradictory.

---

## 3. Impact

### Consequences

- **Developer confusion:** Any developer reading the documentation and writing code based on
  "base is the highest address" will compute addresses incorrectly.
- **Off-by-region errors:** Confusing base and top of a stack can lead to:
  - Stack overflow going undetected (guard page at wrong end).
  - Stack pointer initialization at the wrong address.
  - Memory mapping errors when setting up user-space page tables.
- **Maintenance risk:** Future maintainers may "fix" the code to match the documentation,
  introducing a real bug.

### Severity Assessment

LOW because:
- The code itself is consistent and correct.
- Existing callers presumably follow the implementation, not the documentation.
- No runtime behavior is affected until someone trusts the wrong documentation.

---

## 4. How Verus Verification Discovered This Bug

### The Verification Model

The Verus model for `UserStack` followed the **implementation** semantics:
`top = base + size` (top is the higher address). The Verus consistency report then compared
the model against the source documentation and flagged the mismatch.

### Discovery by AI Reviewer

The exec-consistency report noted:

> "The consistency report correctly identifies a documentation bug in the original kernel:
> comments say base = 'highest address' and top = 'lowest address', but the implementation
> computes `top = base + size` (top > base). The Verus version documents the
> implementation-consistent semantics."

### Classification

This bug was discovered through the verification **consistency checking** process. Building
a formal model requires resolving all ambiguities between documentation and code, which
naturally surfaces cases where the two disagree.

---

## 5. Proposed Fix

Update the documentation to match the implementation:

```rust
///
/// # Description
///
/// Returns the base address of the target stack.
///
/// # Returns
///
/// The base address of the target stack.
///
/// # Notes
///
/// The base address is the lowest address of the stack's memory region.
/// As stacks grow downwards, the stack pointer starts at `top()` (the highest
/// address) and grows toward `base()`.
///
pub fn base(&self) -> PageAligned<VirtualAddress> {
    self.base
}
```

Also fix the typo: "sacks" → "stacks".

---

## 6. File Reference Index

| File | Role |
|------|------|
| `src/kernel/src/mm/ustack.rs` | Original source (doc bug in `base()` comments) |
| `verus/split/kernel/mm/ustack.rs` | Verus exec model (follows implementation semantics) |
| `verus-ai-history/reviews/ustack/exec-consistency_*.md` | Consistency report identifying the divergence |
