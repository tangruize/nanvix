# Review: kredzone (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- **store/load specifications**: Both functions are `external_body` and only assert index validity; `load` returns a placeholder `Ok(0)` with no postcondition relating the returned value to prior `store` calls or the red-zone memory. This leaves the key functional property (read-after-write and preservation of distinct indices) entirely trusted, so the verification does not establish equivalence with the implementation.

### High
- **Missing trusted preconditions**: The stated trust assumptions (T1 size ≥ KREDZONE_SIZE, T2 volatile reads return last write, T3 no concurrency) are not encoded as `requires`/axioms, leaving the external bodies unconstrained. Callers can invoke `store/load` under concurrent or undersized red-zone scenarios without violating any spec, so proofs relying on these assumptions are unsound.

### Medium
- **No ghost/state coupling**: The ghost model (`KernelRedZoneGhost` and `spec_store_effect/spec_load_result`) is not connected to the executable `store/load` contracts. Users must manually maintain ghost state with no spec link, so the model cannot be used to prove that actual calls realize the ghost effects, weakening usability and assurance.

### Low
- _None._

## Positive Observations
- Bounds checking is specified and proven (valid indices, max index, negative indices invalid), matching the implementation’s error handling.
- Abstract algebraic properties of the model (non-interference, commutativity, overwrite, read-after-write) are fully proven over `KernelRedZoneView`.
- Entry size constants are covered with platform lemmas, keeping NUM_ENTRIES consistent with KREDZONE_SIZE.

## Summary
Verification establishes only bounds-checking correctness for the API, while all functional behavior of the red-zone memory is trusted, leaving the essential property (loads return the last stored value at the same index, independent of other indices) unverified. Encode the trust assumptions as preconditions/axioms and strengthen `store/load` specs to relate results to an abstract red-zone state (e.g., by threading ghost state or adding an abstract global) so the proven algebraic properties apply to real calls.
