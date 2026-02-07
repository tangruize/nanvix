# Review: thread_state (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gaps still present (exec/spec/proof)**: The verified module still omits `context_mut`, `fpu_state_mut`, `join_cond`, `impl Debug`, and `impl Drop` from the original `ThreadState` implementation. The update does not add verified stubs or a trusted boundary that preserves the original interface.
  - **Status:** Unresolved.
- **Mutex guard semantics still strengthened beyond implementation (exec/spec/proof)**: `store_mutex_guard` and `take_mutex_guard` still require `!spec_has_mutex` / `spec_has_mutex`, eliminating `BTreeMap::insert` replacement and `remove`-None behavior. There is still no global proof that callers satisfy these preconditions, nor a model of the runtime `Option` path.
  - **Status:** Unresolved.

### Medium
- **Drop safety enforcement still not proven (spec/exec)**: The module only proves that `check_drop_safe()` matches the runtime check; it still does not prove that drop sites satisfy `spec_drop_safe()` (no global invariant or caller proof).
  - **Status:** Unresolved.

### Low
- **Constructor abstraction still omits opaque field initialization (exec/spec)**: `context`, `fpu_state`, and `join_cond` remain unmodeled without spec tokens or ghost state ensuring initialization/preservation.
  - **Status:** Unresolved.

## Positive Observations
- The protocol-level invariants remain internally consistent and well documented.
- Drop-safety equivalence between `check_drop_safe()` and `spec_drop_safe()` is explicit under `wf()`.

## Summary
No substantive fixes were made to the previously reported gaps. The verification remains sound only for the narrowed protocol model, not for the full original interface or runtime behavior.
