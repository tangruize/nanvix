# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: A-

## Verification Result

- **Status**: PASSED (30 verified, 0 errors)
- **Assume statements**: None
- **External bodies**: 7 (all at trust boundaries for ProcessManager/Mutex/Condvar operations)

## Issues Found

### Critical

- None.

### High

- None.

### Medium

1. **Missing coupling between `get_cond` failure and `put_cond` outcome**
   - **Location**: `spec_wait_cond_result` (spec), `put_cond_model` (exec)
   - **Description**: In the original code, when `get_cond` fails, the subsequent
     `put_cond` call will also fail with `NoSuchEntry` (confirmed by inspecting
     `ProcessManager::put_cond` in `src/kernel/src/pm/process/state/mod.rs:382-396`).
     The model treats `get_cond` and `put_cond` outcomes as independent, allowing
     the impossible combination where `get_cond` fails but `put_cond` succeeds.
     This makes `lemma_get_cond_error_propagates` (which assumes all continuation
     steps succeed) vacuously true for that code path. The spec overapproximates
     reality.
   - **Suggested Fix**: Add an optional coupling postcondition to `get_cond_model`
     or a ghost flag that, when `get_cond` fails, implies `put_cond` will also fail.
     Alternatively, document this overapproximation explicitly in the spec comments.
     This doesn't cause unsoundness but weakens the specification.

2. **`spec_cond_ref_released` established without prior reference acquisition**
   - **Location**: `wait_cond_model` postcondition (exec, line 644-645)
   - **Description**: The postcondition `ret.1@.pc matches PcOk ==>
     spec_cond_ref_released(cond_addr as nat)` fires whenever `put_cond` returns
     Ok, even when `get_cond` previously failed and no condvar reference was ever
     acquired. While `spec_cond_ref_released` is uninterpreted (so this doesn't
     cause unsoundness), it's semantically imprecise—"released" implies a prior
     "acquired" that didn't happen. In practice this combination is unreachable
     (see issue #1), but the postcondition doesn't guard against it.
   - **Suggested Fix**: Strengthen the postcondition to condition
     `spec_cond_ref_released` on both `pc == PcOk` AND `gc` being `GcOk` (or at
     minimum `gc` not being an error). This more precisely models the protocol.

3. **Condvar Drop semantics not modeled**
   - **Location**: `get_cond_and_wait_model` (exec), trust boundary documentation
   - **Description**: In the original code, the `Condvar` returned by `get_cond`
     is an `Arc<CondvarInner>` clone. When it goes out of scope at the block end
     (line 129), the Arc refcount decreases. This implicit Drop is important: it
     determines whether `put_cond` will remove the condvar from the map
     (`reference_count() <= 1`). The model collapses the Drop into the trust
     boundary, treating `put_cond_model` as the sole reference release. If the
     Drop were to fail or have side effects beyond refcount decrement (e.g., the
     `CondvarInner::drop` panics when threads are sleeping), the model wouldn't
     capture this.
   - **Suggested Fix**: Document in the trust boundary section that the Condvar
     Drop (Arc refcount decrement) is implicitly absorbed into the
     `get_cond_and_wait_model`→`put_cond_model` transition. Consider adding a
     ghost postcondition on `get_cond_and_wait_model` that the Condvar handle was
     consumed (dropped) before `put_cond` is called.

### Low

1. **Hardcoded ErrorCode literal `22i32` in exec code**
   - **Location**: `wait_cond_model` (exec, line 661)
   - **Description**: The error code `22i32` is hardcoded in the exec function
     rather than using the `ErrorCode::InvalidArgument` constant. While
     `lemma_error_code_matches` proves the spec constant equals
     `ErrorCode::InvalidArgument`, if the `ErrorCode` enum representation changes,
     the exec literal could silently diverge from the spec constant.
   - **Suggested Fix**: Use `ErrorCode::InvalidArgument as i32` directly in the
     exec code, or add a static assertion that pins the value.

2. **Dead `LockTimedOut` variant adds complexity**
   - **Location**: `WaitCondResultModel`, `WaitCondResultView` (exec/spec)
   - **Description**: The `LockTimedOut` variant is provably unreachable
     (`mutex_lock_model` guarantees `!matches!(result, TimedOut)`, proven by
     `lemma_lock_timed_out_unreachable`). Retaining it for exhaustive matching
     is defensible but adds 15+ lines across spec/exec/proof files.
   - **Suggested Fix**: This is a design choice, not a bug. No change required.
     The documentation is adequate.

3. **Uninterpreted resource predicates are not relational**
   - **Location**: `spec_mutex_released`, `spec_mutex_reacquired`,
     `spec_cond_ref_released` (spec)
   - **Description**: Each predicate takes a single address parameter. There's no
     spec-level representation that the mutex and condvar are associated (i.e.,
     that the condvar's wait protocol requires a specific mutex). This is correct
     for this function's scope but limits composability with higher-level
     specifications that need to reason about the condvar-mutex association.
   - **Suggested Fix**: Consider adding a `spec_condvar_associated_mutex(cond_addr,
     mutex_addr)` predicate for future compositional verification. Not needed for
     current scope.

## Positive Observations

- **Comprehensive pipeline modeling**: The `spec_wait_cond_result` spec function
  faithfully captures the multi-step pipeline with stored-result semantics and
  continuation-override behavior. This is a non-trivial control flow pattern
  (stored result + unconditional continuation + `?` override) that is correctly
  modeled.

- **Exec-spec equivalence via ghost state**: The `WaitCondGhostState` mechanism
  elegantly links the exec result to the spec function by capturing all step
  outcomes. This is a clean approach to proving equivalence without
  instrumenting the spec.

- **No assume statements**: All 30 verification conditions pass without any
  `assume` statements. The only trusted elements are the 7 external_body
  functions at well-defined trust boundaries.

- **Thorough error propagation proofs**: Every error path has a dedicated lemma
  (`lemma_take_guard_error_propagates`, `lemma_put_cond_error_propagates`, etc.)
  plus generalized short-circuit lemmas (`lemma_pipeline_short_circuit_timeout`,
  `lemma_take_guard_short_circuit`).

- **Success biconditional**: `lemma_success_requires_all_steps` proves that
  Success holds if and only if ALL pipeline steps return Ok. This is a strong
  correctness property.

- **Resource independence proofs**: The postconditions prove that
  `spec_mutex_released` holds whenever `take_mutex_guard` succeeds,
  independently of later continuation errors. Same for `spec_cond_ref_released`
  and `spec_mutex_reacquired`. This enables reasoning about partial progress.

- **Dead code analysis**: `lemma_lock_timed_out_unreachable` proves the
  `LockTimedOut` variant is dead code (because `mutex_lock_model` with None
  timeout can't produce TimedOut). This is valuable documentation of the
  reacquisition protocol.

- **Well-structured split**: The spec/proof/exec separation is clean. Spec
  defines the abstract model (view types + spec functions), proof proves
  properties of the spec, and exec implements the model and proves equivalence.

- **Clear trust boundary documentation**: The API mapping table and trust
  boundary section explicitly enumerate what is trusted and what is verified.
  The "Properties NOT Proven Here" section is honest about scope limitations.

## Summary

This is a high-quality verification of a complex kernel call with non-trivial
control flow (stored result + unconditional continuation pipeline). The spec
faithfully captures the pipeline semantics, and the 30 verified conditions cover
timeout parsing, error propagation, short-circuit behavior, continuation
override, success biconditional, result exhaustiveness, and resource protocol
predicates. No assume statements are used; trust is confined to 7 external_body
functions at ProcessManager/Mutex/Condvar boundaries.

The main gap is the overapproximation caused by treating `get_cond` and
`put_cond` outcomes as independent (Medium #1), which makes some proved lemmas
vacuously true for impossible input combinations. The Condvar Drop semantics
are also not explicitly modeled (Medium #3), though this is absorbed into the
trust boundary. These are precision issues, not soundness issues—the
verification is sound but could be tighter.

**Recommendation**: Address Medium #1 by adding a coupling predicate or
documentation, and Medium #2 by conditioning `spec_cond_ref_released` on prior
reference acquisition. The remaining issues are low priority and can be deferred.
