# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage still missing for module APIs (exec)**: `ProcessRefMut`, `ProcessRef`, their `state_mut`/`state` accessors, private helpers `get_pmio`/`get_pmio_mut`, and the `Debug` impl remain unverified and are only marked “out of scope” in comments. This does not satisfy the requirement that *all* functions in `src/kernel/src/pm/process/state/mod.rs` have verified counterparts. **Suggested Fix**: Add verified stubs or modeled equivalents for these APIs (even if as pure frame-condition stubs), or explicitly move them into verified scope with faithful signatures.
- **Equivalence gap persists via oracle parameters (exec)**: `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, and `remove_pmio` still rely on caller-supplied booleans/indices (`already_present`, `contains`, `ref_count_at_threshold`, `found`, `found_idx`) with no verified wrapper computing them from an exec model of the collections. The added “oracle parameter” note is documentation only and does not establish semantic equivalence to the original BTreeMap/LinkedList code. **Suggested Fix**: Provide wrapper functions that compute these values from a concrete exec representation (or prove a refinement relation between an exec map/list and the ghost model).

### Medium
- **`external_body` stubs with `ensures true` still lack explicit frame conditions (exec)**: `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `read_pmio_stub`, and `vmem_stub` remain unconstrained; the “&self implies no mutation” comment is not a verified postcondition and does not rule out interior mutability or unsafe side effects. **Suggested Fix**: Add explicit frame-condition ensures for these stubs (matching the other stubs) or justify and prove that these functions cannot mutate ProcessState.
- **Liveness/cleanup still unmodeled in ref-count spec (spec/exec)**: The model still lacks any explicit decrement operation or assumption to reflect dropping clones, so eventual cleanup of mutexes/condvars cannot be proven. The new comment acknowledges this but does not supply a proof obligation or assumption. **Suggested Fix**: Introduce a modeled decrement step or an explicit assumption about external drops, and prove eventual removal properties when counts fall below thresholds.

### Low
- **Return-value identity still abstracted away (exec/spec)**: `get_mutex`/`get_cond` return ghost ref-counts instead of abstract resource tokens, and `remove_pmio` does not model the removed port. This limits reasoning about resource identity/ownership and IO port values. **Suggested Fix**: Add abstract tokens or ghost values for returned resources and relate them to the map/sequence state.

## Positive Observations
- `get_mutex`/`get_cond` now constrain the error path to `spec_mutexes_full()` / `spec_conditions_full()`, fixing the previous over-weak specs.
- `remove_pmio` now requires `found_idx` to be the first occurrence, aligning with `LinkedList::position()` semantics.
- Spec/proof separation remains clean and invariants for counts/capacities are preserved.

## Summary
The update fixes two prior spec weaknesses, but the major gaps remain: several original APIs are still unverified and core functions still depend on oracle parameters without verified wrappers, so semantic equivalence to the original module is not established. Soundness concerns around unconstrained `external_body` stubs and missing liveness modeling persist. Further work is required before this can be considered complete and sound.
