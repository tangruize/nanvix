# Review: process_state (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gap for module-level APIs (exec)**: ProcessRefMut/ProcessRef enums and their `state_mut`/`state` accessors, plus private helpers `get_pmio` and `get_pmio_mut`, and the `Debug` impl are present in the original module but have no verified counterparts. This violates the coverage requirement for all functions in `src/kernel/src/pm/process/state/mod.rs`. **Suggested Fix**: Add verified versions (or explicit out-of-scope stubs with justified specs) for these enums/helpers/impls, or refactor the verified module to include wrappers with faithful signatures.
- **Equivalence gaps due to signature changes and caller-supplied facts (exec)**: Verified functions such as `new`, `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, and `remove_pmio` change signatures or require caller-supplied booleans/indices (`already_present`, `contains`, `ref_count_at_threshold`, `found`, `found_idx`) without any verified wrapper that computes them from actual data structures. This means the verification does not establish semantic equivalence with the original implementation. **Suggested Fix**: Introduce wrapper functions that compute these values from concrete collections (or verified models), or strengthen the specs to tie them to explicit modeled data and prove the wrappers correct.
- **Unjustified `external_body` stubs with unconstrained behavior (exec)**: `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `read_pmio_stub`, and `vmem_stub` have `#[verifier::external_body]` with `ensures true`, which provides no frame conditions and permits arbitrary state changes if called. This is a soundness hole in a core module. **Suggested Fix**: Add explicit frame-condition ensures (as done for other stubs), or wrap them in higher-level verified functions that never rely on their behavior.

### Medium
- **PMIO removal semantics too weak with duplicates (exec/spec)**: `remove_pmio` only requires `found_idx` to point to *some* matching entry, but the original `LinkedList::position()` removes the **first** matching entry. With duplicates, the spec allows removing a later element, breaking equivalence. **Suggested Fix**: Strengthen the precondition to require `found_idx` is the first index where `port_number` occurs.
- **Reference-count model lacks decrements, weakening liveness (spec/exec)**: Ghost ref counts only ever increase in `get_mutex/get_cond` and are removed at thresholds in `put_*`, but no operation models the decrease from dropping clones. This prevents proving eventual cleanup and does not reflect actual `Arc::strong_count()` dynamics. **Suggested Fix**: Add a modeled decrement operation or explicitly justify the abstraction and its limits; add lemmas about eventual removal under drop assumptions.
- **Error conditions for capacity are under-specified (exec/spec)**: `get_mutex/get_cond` specs allow returning `Err(OutOfMemory)` without tying it to `mutex_count/cond_count >= MAX`, so the spec admits spurious failures. **Suggested Fix**: Add ensures that constrain the Err path to the full-capacity condition (matching the original behavior).

### Low
- **Return-value semantics are not modeled (exec/spec)**: `get_mutex/get_cond` return a ghost ref count instead of the `Mutex/Condvar`, and `remove_pmio` returns `Result<(), Error>` rather than the removed `AnyIoPort`. This limits reasoning about resource identity/ownership and IO port values. **Suggested Fix**: Extend the model with abstract tokens or ghost values that represent returned resources and relate them to the map/sequence state.

## Positive Observations
- Well-formedness invariants capture map finiteness, count consistency, capacity bounds, and positive ref counts.
- Capability operations are delegated to a verified `Capabilities` type with wf preservation.
- Mutex/condvar get/put and PMIO add/remove have structured specs with preservation of PID and unrelated state.
- Spec/proof are separated cleanly via `include!`, with focused lemmas in `process_state.proof.rs`.

## Summary
The verification is structured and documents key state-management invariants, but it falls short on coverage and equivalence: several module APIs are missing, signatures differ, and key behaviors are assumed via caller-supplied facts or unconstrained external stubs. Strengthening wrappers, tightening preconditions/ensures, and modeling reference-count decrements would materially improve soundness and fidelity.
