# Review: clock Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### Minor

1. **`pub` fields weaken encapsulation (acknowledged).** The original `TimerTicks` has private fields with `AtomicU32`; the verified model uses `pub minor: u32` and `pub major: u32`. While `lemma_always_wf` proves that `wf()` is universally true (so no invalid states can be constructed), this still permits external code to construct arbitrary `TimerTicks` values bypassing `new()` and `increment()`. The fix report correctly documents this as a Verus requirement for `pub open spec fn` access, and the risk is mitigated by the fact that this is a specification model, not a runtime replacement. Acceptable.

2. **`get()` requires `spec_no_concurrent_writer_assumption()` but original has no such precondition.** The original `get()` is simply `fn get(&self) -> (u32, u32)` with no preconditions. The Verus model adds an uninterpreted predicate as a `requires` clause, which is sound (it makes the contract strictly stronger) but means callers must explicitly obtain the axiom. This is well-documented as Trust Boundary T1 and mechanically enforced. Acceptable.

3. **`now()` returns `(u64, u32)` instead of `SystemTime`.** The type change is necessary since `SystemTime` is an external type. The postcondition `result.1 < 1_000_000_000u32` combined with `spec_system_time_new_succeeds` proves that the original `SystemTime::new()` call would succeed, eliminating the `unreachable!()` path. The modeling is sound.

### Observations

- The `is_max()` and `is_zero()` helpers are verification-only additions not in the original source. They are correctly documented as such and do not affect exec semantics.
- `compute_nanoseconds()` and `compute_seconds()` are refactored out of `now()` for proof decomposition. The arithmetic is identical to the inline expressions in the original.
- `standalone_ticks()`, `standalone_now()`, `now_fallback_model()`, and `now_pit_model()` provide comprehensive coverage of the original API patterns including both `#[cfg]` paths.
- `timer_handler_model()` correctly abstracts the original `timer_handler()` to its only clock-relevant effect (calling `increment()` once), with HAL/scheduler side effects documented as Trust Boundary T3.

## Detailed Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** Five mismatched functions were identified (`new`, `get`, `increment`, `ticks`, `now`), and all have detailed equivalence documentation:

| Function | Divergence | Justification |
|----------|-----------|---------------|
| `new` | `AtomicU32::new(0)` → `u32 = 0` | Sequential model; semantically identical initialization. |
| `get` | `.load(ORDER)` → direct field access | Single-writer assumption (T1); `spec_no_concurrent_writer_assumption()` precondition. |
| `increment` | `wrapping_add(1)` → explicit branching; `&self` → `&mut self` | `lemma_wrapping_add_equiv` proves equivalence (T2). `&mut self` is strictly stronger. |
| `ticks` | `<< 32` → `* 0x1_0000_0000u64` | Mathematical identity (`x << 32 == x * 2^32`). `lemma_shift_eq_mul` documents this. |
| `now` | Returns `(u64, u32)` not `SystemTime`; takes `timer_freq` parameter; factored into helpers; no `unreachable!()` | External type modeling (T4); `#[cfg]` coverage via separate models; proof eliminates panic path. |

### 2. Were MISSING functions added with proper verification?

**Yes.** The `timer_handler` function cannot be directly ported due to `unsafe`, HAL types, global mutable state, and `#[cfg]` blocks. It is modeled by `timer_handler_model()` which:
- Calls `increment()` exactly once (the only clock-relevant effect).
- Has postconditions matching `increment()`'s contract.
- Documents omitted side effects (VM pause check, context switch) as Trust Boundary T3.

The `lemma_timer_handler_single_increment`, `lemma_timer_handler_effect_matches_increment`, and `lemma_timer_handler_side_effects_orthogonal` lemmas formally bridge the behavioral spec to the verified contract.

### 3. Are equivalence justifications sound?

**Yes.** All five equivalence justifications are technically sound:

- **AtomicU32 → u32**: Valid under the documented single-writer assumption. The uninterpreted `spec_no_concurrent_writer_assumption()` makes the trust boundary mechanically visible — Z3 cannot unfold it, so proofs must explicitly propagate it.
- **wrapping_add → branching**: `lemma_wrapping_add_equiv` provides a complete formal proof covering both the `x < u32::MAX` and `x == u32::MAX` cases. The proof uses `nonlinear_arith` for modular arithmetic properties.
- **`<< 32` → `* 0x1_0000_0000`**: A mathematical identity. `lemma_shift_eq_mul` documents it.
- **`#[cfg]` → parameters**: Both cfg paths are covered by `now_fallback_model` (hardcoded `timer_freq = 1`) and `now_pit_model` (parameterized `timer_freq > 0`). `lemma_now_always_valid` is the top-level correctness lemma.
- **`SystemTime` → `(u64, u32)`**: The precondition `nanoseconds < NANOSECONDS_PER_SECOND` is proved by `lemma_nanoseconds_in_range`, which eliminates the `unreachable!()` path.

### 4. Does the exec code now faithfully represent the original source?

**Yes.** Every original function has a corresponding verified model with identical arithmetic and control flow. Structural divergences are limited to Verus modeling constraints (atomics, wrapping, cfg, external types) and are all formally justified. The verification-only additions (`is_max`, `is_zero`, `compute_nanoseconds`, `compute_seconds`, standalone models) extend rather than alter the original semantics.

The torn-read lemmas (`lemma_torn_read_consequence`, `lemma_torn_read_consequence_x86`) go beyond mere consistency restoration — they quantify the consequence of violating the single-writer assumption, which strengthens confidence in Trust Boundary T1.

### 5. Does verification still pass?

**Yes.** Verification passes: 60 verified, 0 errors (9 seconds).

```
verification results:: 60 verified, 0 errors
Duration: 9s
```

## Summary

The clock exec consistency fixes are thorough and well-documented. All five mismatched functions have been properly justified with formal equivalence proofs or documented trust boundaries. The missing `timer_handler` is covered by a verified behavioral model. The verification passes cleanly with 60 verified lemmas and no errors.

The trust boundary documentation is exemplary — five explicitly numbered boundaries (T1–T5) with precise assumptions, and the uninterpreted `spec_no_concurrent_writer_assumption()` predicate makes the concurrency trust boundary mechanically visible in the proof chain. The torn-read consequence lemmas are a valuable addition that quantifies the risk if T1 is violated.

The only reason this is not A+ is the `pub` fields weakening encapsulation, which, while necessary for Verus and mitigated by `lemma_always_wf`, is a departure from the original's privacy guarantees that could matter if the verification model were ever extended with multi-module reasoning.
