# Review: fence Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

- None.

### Minor

1. **`signal()` precondition strengthening is a semantic divergence, not just a model change.**
   The original `signal(&self)` has no guard against over-signaling — `fetch_add(1, Release)` is unconditional. The verified model adds `is_waiting()` (`count < total`) as a precondition. The fix report documents this extensively (including the `kmain.rs` startup fence over-signaling scenario), and the rationale is sound (prevents overflow in the `usize` model, tightens protocol). However, this means the verified model rejects a valid runtime execution path (the last core over-signaling by one). This is acceptable as a deliberate strengthening but should be tracked as a known gap between the verified protocol and runtime behavior.

2. **`wait()` modeled as no-op with satisfaction precondition inverts the original semantics.**
   The original `wait()` *establishes* satisfaction by spinning; the verified `wait()` *requires* it. This is well-documented in the "Trust Boundaries" section and is an inherent limitation of sequential verification. The trust boundary is clearly stated: concurrent signalers are assumed to eventually satisfy the fence. No action needed, but this is the primary verification gap for this module.

3. **Extra functions (`is_satisfied`, `get_count`, `get_total`) lack `#[cfg(verus_keep_ghost)]` or similar gating.**
   These verification-only accessors are harmless and well-documented, but if this code were ever compiled for runtime (which the header disclaims), they would add unnecessary API surface. This is a style nit, not a correctness issue.

## Detailed Analysis

### 1. MISMATCH Functions: Equivalence Assessment

| Function | Original | Verified | Equivalence Sound? |
|----------|----------|----------|--------------------|
| `Fence` (struct) | `count: AtomicUsize, total: usize` | `count: usize, total: usize` (both `pub`) | ✅ Yes — `AtomicUsize` cannot be modeled in Verus; sequential `usize` is the standard approach. `pub` fields are required by Verus `View` trait. |
| `new` | `const fn`, `AtomicUsize::new(0)` | `fn`, `count: 0` | ✅ Yes — `const fn` unsupported in Verus; body is semantically identical (count=0, total=param). Ensures clause properly ties to `FenceView::spec_new`. |
| `wait` | Spin loop with `Acquire` load + `arch::cpu::pause()` | No-op with `is_satisfied()` precondition | ✅ Yes (with trust boundary) — Sequential model cannot express blocking. Precondition captures the postcondition the spin loop establishes. Documented as key verification gap. |
| `signal` | `&self`, `fetch_add(1, Release)` | `&mut self`, `self.count = self.count + 1` | ✅ Yes (with strengthening) — `&mut self` required for Verus mutation. Arithmetic is equivalent. Precondition `is_waiting()` is a deliberate strengthening. |

All equivalence justifications are technically sound.

### 2. EXTRA Functions: Justification Assessment

| Function | Purpose | Justified? |
|----------|---------|------------|
| `is_satisfied` | Exposes satisfaction predicate for ensures/requires clauses | ✅ Yes — needed for proof obligations and test harnesses. Pure observer. |
| `get_count` | Returns signal count for ensures clauses | ✅ Yes — needed for quantitative reasoning in proofs. |
| `get_total` | Returns total for ensures clauses | ✅ Yes — needed for quantitative reasoning in proofs. |

All extra functions are pure observers with no side effects.

### 3. Spec and Proof Quality

- **FenceView** (`fence.spec.rs`): Clean abstract view with `is_satisfied`, `is_waiting`, `remaining`, and `spec_new`. The `#[verifier::ext_equal]` attribute enables extensional equality reasoning. The `inv()` closed spec correctly captures `count <= total`.
- **Proof lemmas** (`fence.proof.rs`): 14 lemmas covering definitional properties, protocol properties, and spec-level concurrency properties. Key highlights:
  - `lemma_state_is_total` + `lemma_satisfied_waiting_complementary`: Prove the state space is a complete partition.
  - `lemma_satisfaction_is_monotone`: Proves once-satisfied-always-satisfied.
  - `lemma_signal_commutativity`: Spec-level proof that signal order doesn't matter (important for concurrent reasoning).
  - `lemma_signals_accumulate_to_satisfaction`: Inductive proof connecting n signals to satisfaction.

### 4. Documentation Quality

The file header documentation is exceptionally thorough:
- "Verification Model" section explains the `AtomicUsize` → `usize` transformation.
- "Verification Scope" explicitly lists what is out of scope (concurrency, liveness, overflow).
- "API Divergence" documents the `signal()` strengthening with concrete code references.
- "Trust Boundaries" identifies the `wait()` semantic inversion as the key gap.
- "Trust Assumptions" lists the overflow assumption.

### 5. Verification Result

```
verification results:: 24 verified, 0 errors
```

All 24 verification conditions pass cleanly.

## Summary

The exec consistency fixes for `fence` are well-executed. All three MISMATCH functions (`new`, `wait`, `signal`) have sound equivalence justifications rooted in fundamental Verus limitations (no atomics, no `const fn`, no interior mutability, no blocking). The three EXTRA functions are legitimate verification-only accessors. The documentation is exemplary — particularly the "API Divergence" section's detailed analysis of the `signal()` over-signaling scenario in `kmain.rs`. The proof suite is comprehensive with 14 lemmas covering definitional, protocol, and concurrency properties. The only substantive concern is that the `signal()` precondition strengthening and the `wait()` semantic inversion represent real verification gaps for concurrent correctness, but these are clearly documented as intentional trust boundaries. Verification passes cleanly with 24/24 conditions verified.
