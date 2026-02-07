# Review: thread_state (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **API Mismatch in Mutex Operations**: The verified `store_mutex_guard` and `take_mutex_guard` functions do not match the original API signature or semantics regarding resource ownership.
  - Original: `store_mutex_guard` takes a `MutexGuard` (ownership transfer) and `take_mutex_guard` returns `Option<MutexGuard>`.
  - Verified: `store_mutex_guard` takes only a ghost address and drops the guard; `take_mutex_guard` returns nothing.
  - **Impact**: The verified code cannot replace the original because it fails to store/return the actual lock guards, breaking the RAII locking mechanism. It verifies the *accounting* of locks but not the *holding* of locks.
  - **Fix**: Define a trusted `MutexGuard` wrapper type (even if opaque) in the verified code and include it in the `store`/`take` signatures to preserve data flow, or explicitly document this as a "protocol-only" model that ignores resource payloads.

### Medium
- **Implementation Divergence (Map vs Counter)**: The verified `ThreadState` uses a `locked_mutex_count: usize` field, whereas the original uses `locked_mutexes: BTreeMap`.
  - The verification proves that a counter-based implementation satisfies the drop-safety checks.
  - It does *not* verify that the actual `BTreeMap` implementation works or that the original code's `!is_empty()` check is correct (though they are logically equivalent under the proven invariants).
  - **Fix**: Acknowledge this abstraction gap. Ideally, model `locked_mutexes` as a `Map` in exec code if possible, or maintain the `locked_mutex_count` as a shadow field if this code is meant to be a model.

- **Missing Function Coverage**: Several public/internal methods are missing from the verified module:
  - `context_mut()`
  - `fpu_state_mut()`
  - `join_cond()`
  - **Impact**: The verification covers only the "metadata" part of the thread state, not the execution context or synchronization primitives.
  - **Fix**: Add these methods, even if they return opaque/unverified types, to ensure the verified struct can fully replace the original in terms of API surface.

### Low
- **Missing Struct Fields**: The verified struct omits `context`, `fpu_state`, and `join_cond`.
  - While acceptable for a partial verification, this prevents the verified struct from being used as a drop-in replacement.
  - **Fix**: Add these fields as opaque types (e.g., using `external_type!`) to the verified struct to match the memory layout and capability of the original.

## Positive Observations
- **Strong Spec/Proof Separation**: The split between `state.rs`, `state.spec.rs`, and `state.proof.rs` is clean and follows best practices.
- **Correct Resource Modeling**: The use of `Option<int>` tokens with `take` semantics effectively captures the resource linearity of the kernel/user stacks without getting bogged down in implementation details.
- **Sound Invariants**: The `wf()` predicate correctly relates the ghost set to the runtime counter (in the model), ensuring that `check_drop_safe` is provably correct within the model.
- **No Unjustified Assumptions**: The core module is free of `assume` or `external_body` shortcuts (excluding the implicit trust in the model mapping).

## Summary
The verification of `thread_state` is a **high-quality model** of the state management protocol, specifically proving that resource tracking (stacks) and lock accounting (mutexes) are consistent and drop-safe. However, it is **not** a code-level verification of the actual `ThreadState` implementation. It diverges significantly in data structures (Counter vs Map) and API (dropping MutexGuards), meaning the verified code cannot currently replace the original source. It serves as a formal proof of the *design* of the state tracking, but not the *implementation*.
