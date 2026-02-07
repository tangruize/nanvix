# Review: thread_state (claude-opus-4.6)

## Grade: B+

## Verification Result

- **Status**: PASSED
- **Verified items**: 37
- **Errors**: 0
- **No `assume`, `external_body`, or `trusted`** in the module.

## Issues Found

### Critical

_(none)_

### High

- **Location**: `store_mutex_guard()` (exec: state.rs:227-241), `spec_locked_mutex_count` (spec: state.spec.rs:79)
- **Description**: The mutex guard abstraction models a `BTreeMap<MutexAddress, MutexGuard>` as a plain `locked_mutex_count: usize` counter. This loses key-based semantics. In the original, `BTreeMap::insert()` overwrites an existing entry (map size unchanged when re-locking the same address), whereas the verified model unconditionally increments the count. This means:
  - `store_mutex_guard(addr=A); store_mutex_guard(addr=A)` → original: 1 mutex locked; model: 2 counted.
  - `take_mutex_guard(addr=B)` when only addr=A is locked → original: returns `None`; model: returns `true` and decrements.
  - The postcondition `!self.spec_drop_safe()` (line 232) is always asserted after store, but in the original, re-inserting the sole locked mutex keeps the map at size 1 — the thread could still be drop-safe after a "redundant" lock.
- **Suggested Fix**: Model locked mutexes as a `Map<int, bool>` (ghost map from abstract mutex address to presence). Change `store_mutex_guard` to take an abstract address parameter and use `map.insert(addr, true)`; change `take_mutex_guard` to take an address and return `map.contains_key(addr)`. This faithfully captures insert-or-update and per-key removal semantics. Alternatively, if the kernel guarantees no double-locking (which is true in practice — double-locking is a deadlock), document this as a precondition: `requires !self.spec_has_mutex(address)`.

### Medium

- **Location**: `wf()` (spec: state.spec.rs:88-89)
- **Description**: The well-formedness predicate is trivially `true`. Every lemma that ensures `post.wf()` is vacuously proven, and every `requires self.wf()` precondition is vacuously satisfied. This means `wf()` provides no meaningful structural invariant. The proof infrastructure (7 wf-preservation lemmas) is pure boilerplate that verifies nothing.
- **Suggested Fix**: Add meaningful constraints. At minimum: `self.id.spec_value() >= 0` (ThreadIdentifiers in Nanvix should be non-negative for valid threads) or constraints relating stack presence to thread type. If no meaningful invariant exists, remove the `wf()` predicate and associated lemmas to reduce noise, or document explicitly why it is intentionally left trivial for future extension.

- **Location**: `lemma_interrupt_reason_roundtrip()` (proof: state.proof.rs:218-233)
- **Description**: The ensures clause is vacuously true for the `Some` case:
  ```
  post.spec_interrupt_reason() == self.spec_interrupt_reason()
      || self.spec_interrupt_reason().is_some()
  ```
  When `self` already has an interrupt reason (`is_some()`), the second disjunct is trivially true regardless of `post`. The lemma claims to prove a "round-trip" but actually proves nothing useful when the thread was already interrupted. A proper round-trip property would show that the value passed to `set` is the value returned by `take`.
- **Suggested Fix**: Replace with a stronger lemma that directly states the round-trip property:
  ```rust
  pub proof fn lemma_interrupt_reason_roundtrip(&self, reason: int)
      ensures ({
          let mid = ThreadState { interrupt_reason: Some(reason), ..*self };
          let post = ThreadState { interrupt_reason: None, ..mid };
          post.spec_interrupt_reason().is_none()
          && mid.spec_interrupt_reason() == Some(reason)
          && post.spec_id() == self.spec_id()
      }),
  ```
  The existing `lemma_interrupt_reason_set_take_on_none` (lines 236-252) partially covers this but only for the `!is_interrupted()` case. A complete round-trip lemma should work unconditionally.

### Low

- **Location**: `context_mut()`, `fpu_state_mut()`, `join_cond()` — not present in verified code
- **Description**: Three original functions are omitted from verification. These return raw pointers (`*mut ContextInformation`, `*mut FpuState`) or cloned sync primitives (`Condvar`). The omission is documented and justified (opaque HAL/sync boundary types that cannot be meaningfully modeled in pure spec), but it means raw pointer safety for context/FPU access is unverified.
- **Suggested Fix**: No action required for the current scope. For future work, consider adding `external_body` stubs with at least frame-condition specs (e.g., "calling `context_mut()` does not modify any ThreadState fields") so callers can reason about state preservation.

- **Location**: `Drop::drop()` — not present in verified code (original: state.rs:283-290)
- **Description**: The original `Drop` implementation logs an error if locked mutexes remain. The verification models this property via `spec_drop_safe()` but does not verify the drop implementation itself. The connection between the spec predicate and actual runtime behavior is unverified.
- **Suggested Fix**: Add a comment in the exec file explicitly noting that `spec_drop_safe()` is the verification-side encoding of the Drop invariant. No exec verification is needed since the Drop only logs (no correctness impact), but the relationship should be documented.

- **Location**: `spec_has_resources()` (spec: state.spec.rs:106-108)
- **Description**: This spec function is defined but never referenced in any exec function postcondition, precondition, or proof lemma. It is dead specification code.
- **Suggested Fix**: Either remove it or add a lemma/usage that justifies its existence (e.g., a lemma proving that after both `take_kernel_stack` and `take_user_stack`, `!spec_has_resources()`).

- **Location**: `fmt::Debug` trait impl — not present in verified code (original: state.rs:276-280)
- **Description**: The `Debug` formatting implementation is not verified. This is a display-only function with no correctness impact.
- **Suggested Fix**: None needed. Reasonable omission.

## Positive Observations

- **Clean verification**: 37 items verified, 0 errors, no `assume`/`external_body`/`trusted` in the module. The verification is entirely self-contained and sound within its abstraction boundary.
- **Thorough frame conditions**: Every mutating function specifies that all unrelated fields are preserved (e.g., `self.spec_id() == old(self).spec_id()` in every mutator). This is excellent practice that prevents accidentally losing state.
- **Well-documented abstractions**: The exec file header (lines 1-46) clearly documents what is modeled, what is abstracted, what is out of scope, and why. This makes the verification's trust boundary explicit.
- **Good split structure**: Spec, proof, and exec are cleanly separated. The spec file defines the abstract view type and spec functions; the proof file contains only lemmas; the exec file contains only implementations with contracts.
- **Option take/store semantics are correct**: The `take_kernel_stack`, `take_user_stack`, `take_interrupt_reason`, `set_interrupt_reason`, `store_thread_data_area`, and `get_thread_data_area` functions all correctly model their original Option-based semantics.
- **Drop safety property**: The `spec_drop_safe()` predicate captures the essential safety invariant (no locked mutexes at destruction), and construction is proven to produce drop-safe state.
- **Comprehensive proof lemmas**: 20+ proof lemmas cover construction, ID immutability, Option semantics, mutex count, well-formedness preservation, round-trip properties, and view equality.

## Summary

This is a solid verification of the ThreadState management protocol. The core state operations (Option take/store, field accessors, interrupt reason management) are faithfully modeled and thoroughly specified with proper frame conditions. The verification is clean — no assumptions or trusted code — and well-documented.

The main weakness is the **mutex guard count abstraction**, which substitutes a `BTreeMap` with a plain counter and thereby loses per-key semantics. This means the spec does not capture the difference between locking a new mutex vs. re-locking an existing one, and take operations cannot distinguish between different mutex addresses. While this may be acceptable if the kernel guarantees no double-locking (as would be the case in practice, since double-locking causes deadlock), the gap should be either closed with a map-based model or explicitly documented with a precondition.

The trivially-true `wf()` predicate and the vacuous round-trip lemma are secondary concerns that reduce the verification's value without causing unsoundness. Strengthening these would raise the grade to an A-.

**Recommendations (priority order)**:
1. Model locked mutexes as a ghost `Map<int, bool>` or add a no-double-lock precondition to `store_mutex_guard`.
2. Either strengthen `wf()` or remove it and its associated boilerplate lemmas.
3. Fix `lemma_interrupt_reason_roundtrip` to state a non-vacuous property.
4. Remove or use `spec_has_resources()`.
