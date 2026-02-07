# Review: thread_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Core API coverage still incomplete**: `context_mut`, `fpu_state_mut`, `join_cond`,
  `fmt::Debug::fmt`, and `Drop::drop` remain omitted. The model still elides
  `context`, `fpu_state`, and `join_cond`, so constructor behavior and invariants
  for these resources are unverified. Documentation alone does not close this gap.
- **Mutex guard removal semantics still stronger than original**: `take_mutex_guard`
  continues to require `spec_has_mutex(address@)` and returns no `Option`, removing
  the not-found path that exists in `BTreeMap::remove`. This can mask release-of-
  unheld mutex bugs and is not behaviorally equivalent.

### Medium
- **Guard payload/replacement not modeled**: `store_mutex_guard` still forbids
  re-insertion and models only an address set. The replacement behavior and guard
  identity/ownership from `BTreeMap::insert` are not represented, so RAII payload
  semantics remain outside the model.

### Low
- None.

## Positive Observations
- Drop-safety equivalence is now explicit: `check_drop_safe`, count, and ghost-set
  emptiness are proven equivalent under `wf()`, improving the prior modeling gap.
- Specs and documentation are clearer about the protocol-only model and frame
  conditions.

## Summary
The update clarifies intent and tightens drop-safety reasoning, but the key
coverage gaps and strengthened mutex-release semantics remain. Verification is
therefore still incomplete relative to the original implementation.
