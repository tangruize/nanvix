# Review: kcall_create_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- None.

### Medium

1. **Trivially true lemma: `lemma_copy_source_matches_validated_address`**
   - **Location**: `create_thread.proof.rs`, line 424–434
   - **Description**: The ensures clause is `input.arg0 == input.arg0`, which is a tautology and proves nothing. The intent (documenting that step 1 validation address and step 2 copy source address are the same `arg0`) is valuable, but it is expressed only in comments, not in the proof obligation. This lemma provides zero verification value — a reader might mistakenly believe address linkage is mechanically proven when it is only documented.
   - **Suggested Fix**: Either (a) remove the lemma and convert it to a doc comment in the exec code, or (b) strengthen it to prove a meaningful property, e.g., by adding a precondition constraining `input.arg0` to equal some externally-provided copy source address parameter: `requires input.arg0 == copy_src_addr, ensures copy_src_addr == input.arg0`. Even this is modest — ideally, the ghost state threading should be leveraged to prove the `copy_from_user` ghost `src_addr` parameter equals the `is_user_region` ghost `addr` parameter from step 1, which is currently only established by construction.

2. **`spec_user_stack_valid` merges two distinct validation steps**
   - **Location**: `create_thread.spec.rs`, line 183–184
   - **Description**: The original code performs two separate checks: (1) `is_user_region(user_stack_base, user_stack_size)` and (2) `user_stack_size < USER_STACK_SIZE`. These are distinct failure modes with different log messages. The spec merges them into a single predicate `spec_user_stack_valid = user_stack_valid && user_stack_size >= USER_STACK_SIZE()`. While the exec model correctly separates them into Steps 4 and 4b, the spec conflation means proof lemmas cannot distinguish the two failure reasons. For example, `lemma_user_stack_invalid_propagates` fires on either failure but cannot state which check failed.
   - **Suggested Fix**: Split into `spec_user_stack_region_valid(input)` and `spec_user_stack_size_valid(input)`, and add separate propagation lemmas for each. Update `spec_all_validations_passed` to use both predicates.

3. **External body `is_user_region`/`is_user_addr` accept pre-computed boolean — no oracle verification**
   - **Location**: `create_thread.rs`, lines 391–419
   - **Description**: The external bodies take the answer (`valid: bool`) as an input parameter and simply return it. The verification proves dispatch correctness *given* correct validation outcomes but does NOT prove the validation outcomes are correct. This is acknowledged in the trust boundary documentation but represents the largest soundness gap in this module. A caller could supply `valid = true` for an address that does not lie in user space, and the model would accept it.
   - **Suggested Fix**: This is an inherent limitation of the model-based architecture. The recommended mitigation is to add cross-module linking lemmas that connect the VMM module's verified `is_user_region`/`is_user_addr` specs to the boolean inputs used here. A comment linking to the VMM module's verification would improve traceability.

### Low

1. **Bridge functions are never integrated into tests**
   - **Location**: `create_thread.rs`, lines 524–557
   - **Description**: `assert_thread_create_args_size()` and `assert_user_stack_size()` are provided as build-time bridge functions to verify spec constants match runtime values, but no integration test or build-time assertion actually calls them. If `ThreadCreateArgs` layout or `USER_STACK_SIZE` changes, the spec constants silently drift.
   - **Suggested Fix**: Add a `#[test]` in `src/libs/sys/` or `src/tests/` that calls the equivalent assertions: `assert_eq!(size_of::<ThreadCreateArgs>(), 28)` and `assert_eq!(config::memory_layout::USER_STACK_SIZE, 524288)`. Alternatively, add `static_assert!` to the original source.

2. **Architecture-dependent `u32` assumption**
   - **Location**: `create_thread.rs`, `ThreadCreateArgsModel` (lines 281–302)
   - **Description**: All address and size fields use `u32`, matching x86-32's 32-bit `usize`. This is documented (line 278–280) but is a hardcoded architecture dependency. If Nanvix ever targets a 64-bit architecture, `ThreadCreateArgsModel`, `THREAD_CREATE_ARGS_SIZE()`, and `USER_STACK_SIZE()` all need updating.
   - **Suggested Fix**: Add a comment in the spec constants referencing a single architecture-configuration point, or define a spec type alias for the address width. Acceptable as-is given Nanvix currently targets only x86-32.

3. **`IRRELEVANT_PM_OUTCOME` uses technically invalid error code**
   - **Location**: `create_thread.spec.rs`, line 316
   - **Description**: The sentinel value `CtError { error_code: 0 }` violates `spec_is_valid_error_code` (which requires `code > 0`). While `lemma_short_circuit_on_validation_failure` proves this value is never observed in the final result, using an invalid sentinel in ghost code is slightly misleading.
   - **Suggested Fix**: Use `CtOk { tid: 0 }` or add a comment explicitly noting the value is intentionally invalid to signal "not used."

4. **`spec_is_error_code_value` covers only a subset of kernel ErrorCode variants**
   - **Location**: `create_thread.spec.rs`, lines 299–306
   - **Description**: The predicate enumerates only 6 of the ~30+ `ErrorCode` variants in the full kernel. While the doc comment explains this, any future PM/VMM call site returning a non-listed error code would not satisfy `spec_is_error_code_value`. The broader `spec_is_valid_error_code(code > 0)` is used in external body postconditions, which is correct but weaker.
   - **Suggested Fix**: Acceptable as-is. Consider adding a comment linking to the full `ErrorCode` enum in `src/libs/error/src/lib.rs` for maintainability.

## Positive Observations

- **No `assume` statements**: All three files (exec, spec, proof) are free of `assume!` macros, meaning no verification obligations are bypassed.
- **Comprehensive documentation**: Trust boundaries (T1–T6) are explicitly enumerated with detailed explanations. The API mapping table provides clear traceability between original and model functions.
- **Clean spec/proof/exec separation**: Specifications, proof lemmas, and executable model are properly factored into three files with clear responsibilities.
- **Ghost parameter threading**: Argument identity is tracked through the pipeline via ghost parameters (`ghost_pid`, `ghost_arg0`, `ghost_user_fn_addr`, etc.), proving that the same concrete values flow through validation and PM call steps.
- **Biconditional success lemma**: `lemma_success_requires_all_steps` proves success ⟺ (all validations pass ∧ PM succeeds), which is the strongest correctness statement for the pipeline.
- **Short-circuit proof**: `lemma_short_circuit_on_validation_failure` formally proves that early validation failures make the PM outcome irrelevant — an important correctness property for a multi-step pipeline.
- **Error code linkage**: `lemma_error_code_matches` ties the spec constant to the concrete `ErrorCode::InvalidArgument` discriminant, and `lemma_copy_error_code_valid` proves that copy error codes satisfy the validity predicate.
- **All 21 verification conditions pass** with zero errors.
- **Out-of-scope properties explicitly documented**: The module clearly states which properties (schedulability, TID uniqueness, stack mapping, resource cleanup) are verified elsewhere.

## Summary

This is a well-executed model-based verification of the `create_thread` kernel call validation pipeline. The verification covers all six pipeline stages (address validation, copy-from-user, user_fn check, stack check, TDA check, PM call) with correct error propagation semantics. The proof suite is thorough: each error path has a dedicated propagation lemma, the success path is proven biconditionally, result exhaustiveness and mutual exclusivity are established, and short-circuit behavior is formally verified.

The main limitations are inherent to the model-based approach: (1) validation oracles are trusted (booleans passed in rather than computed), and (2) the model operates on abstract types rather than concrete kernel types. These are well-documented design choices consistent with the project's per-module verification architecture. The trivially true `lemma_copy_source_matches_validated_address` is the only lemma that should be revised or removed, as it creates an illusion of proof where only a tautology exists.

Recommendations: (1) fix or remove the tautological lemma, (2) split `spec_user_stack_valid` for finer-grained reasoning, and (3) add build-time integration tests for the bridge functions to prevent spec constant drift.
