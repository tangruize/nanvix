# Review: thread_state (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gaps still present (exec/spec/proof)**: The verified module still omits `context_mut`, `fpu_state_mut`, `join_cond`, `impl Debug`, and `impl Drop` from the original `ThreadState` implementation. The new documentation explicitly says these are out of scope, but there are still no verified stubs or trusted boundaries that preserve the interface. This remains incomplete coverage, not a functional fix.
  - **Status:** Unresolved (documentation only).
- **Mutex guard semantics still strengthened beyond implementation (exec/spec/proof)**: `store_mutex_guard` and `take_mutex_guard` still require `!spec_has_mutex` / `spec_has_mutex`, eliminating the `BTreeMap::insert` replacement and `remove`-None behavior present in the runtime code. The new “T1/T2 trust assumptions” acknowledge this, but there is still no global proof that callers satisfy them, nor a model of the runtime `Option` path.
  - **Status:** Unresolved (assumed, not proven).

### Medium
- **Drop safety enforcement still not proven (spec/exec)**: The new comments explicitly say that enforcement of `spec_drop_safe()` at drop sites is out-of-scope, but there is still no proof that drop sites satisfy it. The module only proves the check is equivalent to the runtime check, not that the check always passes.
  - **Status:** Unresolved (acknowledged).

### Low
- **Constructor abstraction still omits opaque field initialization (exec/spec)**: The constructor continues to ignore `context`, `fpu_state`, and `join_cond` entirely. The documentation now calls this out, but there is still no spec-level token/ghost state to represent their initialization or preservation.
  - **Status:** Unresolved (documentation only).

## Positive Observations
- The added trust-scope documentation is clearer and makes the intended boundary explicit.
- The protocol-level invariants (ID immutability, Option take/set, mutex accounting) remain consistent and well-scoped.
- Drop-safety equivalence between `check_drop_safe()` and `spec_drop_safe()` is clearly documented and proved under `wf()`.

## Summary
The update adds helpful documentation but does not actually resolve the previously reported gaps. Coverage is still incomplete, stronger mutex-guard preconditions remain unproven globally, and drop-safety enforcement is not established. The verification is internally consistent for the modeled protocol but not complete or fully sound for the original interface.
