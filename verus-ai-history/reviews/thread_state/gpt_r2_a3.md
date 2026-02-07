# Review: thread_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Core API coverage still incomplete**: `context_mut`, `fpu_state_mut`,
  `join_cond`, `fmt::Debug::fmt`, and `Drop::drop` remain omitted. The model
  continues to elide `context`, `fpu_state`, and `join_cond`, so construction
  behavior and invariants for these resources remain unverified.
- **Mutex guard removal semantics still stronger than original**:
  `take_mutex_guard` still requires `spec_has_mutex(address@)` and returns no
  `Option`, eliminating the not-found path from `BTreeMap::remove` and masking
  release-of-unheld mutex bugs.

### Medium
- **Guard payload/replacement not modeled**: `store_mutex_guard` still forbids
  re-insertion and models only an address set. The replacement behavior and RAII
  guard identity/ownership from `BTreeMap::insert` remain outside the model.

### Low
- None.

## Positive Observations
- Drop-safety equivalence between `check_drop_safe`, the counter, and the ghost
  set remains explicitly documented and proven under `wf()`.
- Specs and frame conditions remain clear and consistent with the protocol-only
  model.

## Summary
The updated files do not address the prior coverage and semantics gaps. The
verification remains incomplete relative to the original implementation.
