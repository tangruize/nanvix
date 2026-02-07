# Review: thread_state (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gaps (exec/spec/proof)**: Verified module omits `context_mut`, `fpu_state_mut`, `join_cond`, `impl Debug`, and `impl Drop` from the original `ThreadState` (src/kernel/src/pm/thread/state.rs). These functions/impls have no verified counterparts, so coverage is incomplete.
  - **Suggested Fix:** Add verified exec/spec/proof stubs that model the behavior of these functions (e.g., opaque-return specs for `context_mut`/`fpu_state_mut`, a modeled `join_cond` clone, and a spec/proof of `Drop`’s check/log behavior), or explicitly justify and fence them with trusted boundaries in the module interface.
- **Mutex guard semantics strengthened beyond implementation (exec/spec/proof)**: `store_mutex_guard` and `take_mutex_guard` in the verified exec add preconditions `!spec_has_mutex` and `spec_has_mutex` (state.rs), which rule out `BTreeMap::insert` replacement and `remove`-returns-None behaviors present in the original. This is a stronger contract than runtime behavior and is not discharged by proofs.
  - **Suggested Fix:** Either (a) model the `Option` return path and replacement semantics explicitly in spec/proof, or (b) add verified caller-side invariants proving no double-locking and release-only-held across the whole kernel, and document this as a required global invariant.

### Medium
- **Drop safety property is only a check, not an enforced invariant (spec/exec)**: `spec_drop_safe` + `check_drop_safe` prove equivalence to the `Drop::drop()` emptiness check, but there is no proof that thread state is drop-safe at all drop sites (i.e., no global invariant or proof that locked mutexes are always released before drop).
  - **Suggested Fix:** Add a module-level invariant or protocol proof that `spec_drop_safe()` holds whenever a `ThreadState` is dropped, or enforce it by construction in all verified callers.

### Low
- **Constructor abstraction omits initialization of opaque fields (exec/spec)**: Verified `new` drops `context`, `fpu_state`, and `join_cond` initialization from the model. While documented as out-of-scope, this leaves no spec-level guarantee that these fields are properly initialized or preserved.
  - **Suggested Fix:** Add abstract spec fields or ghost tokens that represent initialization/ownership of these opaque fields, or add explicit trusted boundary notes where these fields are ignored.

## Positive Observations
- The verification cleanly separates exec/spec/proof, with clear documentation of the abstraction boundary and trust assumptions.
- `wf()` ties the ghost mutex set to the runtime counter, and non-interference lemmas capture per-address `BTreeMap` semantics well.
- Option take/set semantics, ID immutability, and drop-check equivalence are explicitly specified and proven.
- No `assume`/`external_body` shortcuts were found in the core module.

## Summary
The verification is strong for the modeled protocol (ID immutability, option semantics, mutex accounting, and drop-check equivalence), but it falls short on full coverage and on ensuring the strengthened mutex preconditions are justified globally. To reach a higher grade, add verified counterparts for all original functions/impls and either model or prove the no-double-lock/release-only-held invariants across callers. With these gaps addressed, the split structure and invariants should support a comprehensive correctness argument.
