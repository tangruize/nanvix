# Review: kcall_wait_cond (claude-opus-4.6)

## Grade: A

## Verification Result

- **Status**: PASSED (30 verified, 0 errors)
- **Assume statements**: None
- **External bodies**: 7 (all at trust boundaries for ProcessManager/Mutex/Condvar operations)

## Previous Issue Resolution

### Medium #1: Missing coupling between `get_cond` failure and `put_cond` outcome
- **Status**: ADDRESSED via documentation
- **Verification**: The trust boundary documentation (T4, lines 75-79 of
  wait_cond.rs) now explicitly acknowledges the overapproximation: "the model
  treats get_cond (T2) and put_cond (T4) outcomes as independent. In practice,
  when get_cond fails, put_cond will also fail... This is sound
  (overapproximate) but imprecise; the coupling is a PM-internal refcount
  invariant outside this module's scope." This was one of my suggested fixes
  (document the overapproximation). The other suggestion (adding a coupling
  predicate) would have been more precise but is a cross-module concern.
  Acceptable resolution.

### Medium #2: `spec_cond_ref_released` established without prior reference acquisition
- **Status**: FIXED
- **Verification**: The postcondition was strengthened from
  `ret.1@.pc matches PcOk ==> spec_cond_ref_released(...)` to
  `(ret.1@.gc matches GcOk && ret.1@.pc matches PcOk) ==> spec_cond_ref_released(...)`
  (exec lines 656-658). This correctly conditions the predicate on prior
  reference acquisition. Additionally, all early-exit ghost state paths now use
  Error variants for don't-care fields (e.g., `GcError { error_code: 0 }`
  instead of the previous `GcOk`), preventing false triggering. This is a
  thorough fix that addresses both the postcondition and the ghost state
  consistency.

### Medium #3: Condvar Drop semantics not modeled
- **Status**: ADDRESSED via documentation
- **Verification**: Trust boundary T3 (lines 69-72) now documents: "The Arc
  clone from T2 is implicitly dropped at the end of the block containing
  T2+T3, decrementing the refcount. This Drop is absorbed into the T2→T4
  trust boundary transition." This explicitly acknowledges the implicit Drop
  as part of the trust boundary, which was the suggested fix.

### Low #1: Hardcoded ErrorCode literal `22i32`
- **Status**: FIXED
- **Verification**: Changed to `let error_code: i32 = ErrorCode::InvalidArgument as i32;`
  (exec line 670), then used as `WaitCondResultModel::InvalidTimeoutError { error_code }`.
  If the enum repr changes, compilation will catch the mismatch. Clean fix.

### Low #2: Dead `LockTimedOut` variant
- **Status**: Retained (design choice) — acceptable.

### Low #3: Uninterpreted predicates not relational
- **Status**: Not addressed — acceptable (future enhancement suggestion).

## New Issues Check

### Were any new issues introduced by the fixes?

1. **Ghost state don't-care values changed from Ok to Error variants**: All
   early-exit paths now use Error variants for unreached steps (e.g.,
   `TmgError { error_code: 0 }` instead of `TmgOk`). I verified each path:
   - Invalid timeout: All 7 fields use non-Ok variants. ✓
   - TakeMutexGuard error: `tmg` is actual view; remaining 6 use non-Ok. ✓
   - PutCond error: `tmg`, `gc`, `cw`, `pc` are actual; remaining 3 use non-Ok. ✓
   - GetMutex error: `tmg`-`gm` actual; `lo`, `pg` use non-Ok. ✓
   - Lock error paths: `pg` uses `PgOk` — this is acceptable since the `lo`
     view already captures the error, and the spec_wait_cond_result
     short-circuits at the lock match before reaching put_guard. ✓
   No new issues from this change.

2. **`USIZE_BITS() == 32` precondition added to `wait_cond_model`** (line 628):
   This was previously only an implicit assumption. Making it explicit is
   strictly an improvement — it documents the architecture requirement and
   forces callers to establish it.

No new issues found.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- None. All previous medium issues have been adequately addressed.

### Low

1. **Dead `LockTimedOut` variant adds complexity** (retained from R1)
   - **Location**: `WaitCondResultModel`, `WaitCondResultView` (exec/spec)
   - **Description**: Provably unreachable variant retained for exhaustive
     matching. This is a deliberate design choice, adequately documented and
     proven dead by `lemma_lock_timed_out_unreachable`.
   - **Status**: Acceptable, no action required.

2. **Uninterpreted resource predicates are not relational** (retained from R1)
   - **Location**: `spec_mutex_released`, `spec_mutex_reacquired`,
     `spec_cond_ref_released` (spec)
   - **Description**: Single-address predicates don't capture the
     condvar-mutex association. Correct for this module's scope but limits
     compositional verification.
   - **Status**: Future enhancement, no action required for this module.

## Positive Observations

All previous positive observations remain valid, plus:

- **Careful ghost state engineering**: The don't-care ghost values were
  systematically changed from Ok to Error variants to be consistent with
  the new resource-release postconditions. This shows disciplined reasoning
  about how ghost state interacts with postconditions — a common source of
  subtle bugs in Verus proofs.

- **Well-documented trust boundary overapproximation**: The acknowledgment
  that get_cond and put_cond outcomes are treated as independent (with
  explanation of why the coupling is a PM-internal invariant) is excellent
  practice. It tells future verifiers exactly what is overapproximate and
  why tightening it would require cross-module verification.

- **Strengthened postconditions**: The addition of per-step resource release
  postconditions (tmg→mutex_released, gc+pc→cond_ref_released,
  lo→mutex_reacquired) enables callers to reason about partial progress
  even when the overall call fails. This is a meaningful improvement over
  the previous version which only asserted resource properties on
  stored-result returns.

- **Architecture guard made explicit**: The `USIZE_BITS() == 32`
  precondition on `wait_cond_model` makes the x86-32 requirement explicit
  rather than implicit, improving proof composability.

## Summary

The prover addressed all six issues from the previous review. Three medium
issues were resolved: #1 and #3 via explicit trust boundary documentation
(sound overapproximation acknowledgment), #2 via a proper postcondition
strengthening that conditions `spec_cond_ref_released` on `GcOk && PcOk`.
Low #1 was fixed by using `ErrorCode::InvalidArgument as i32` instead of a
hardcoded literal. The remaining low issues were design choices or future
enhancements, correctly left unchanged.

The fixes introduced no new issues. Ghost state don't-care values were
carefully updated to use Error variants, preventing false triggering of the
new resource-release postconditions. The verification still passes with 30
verified conditions and 0 errors, with no assume statements.

This is now a solid A-grade verification: correct, well-documented, with
explicitly acknowledged trust boundaries and overapproximations. The only
remaining items are minor design preferences (Low #1, #2) that don't affect
soundness or practical utility.
