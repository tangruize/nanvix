# Re-Review: ready (claude-opus-4.6)

## Grade: A

## Previous Issue Verification

### High: Boundary models not cross-validated — VERIFIED FIXED

**Claim:** Added `CROSS-MODULE-CHECK` annotations.
**Evidence:** Confirmed at lines 170–177 (`RunningThread::from_state`) and lines 211–217 (`ZombieThread::from_state`). Each annotation enumerates the specific postconditions (`spec_id`, `spec_is_interrupted`, `spec_locked_mutex_count`, `spec_has_mutex`, `spec_drop_safe`, `wf()`) that must be validated against the real module. Searchable via `grep -r CROSS-MODULE-CHECK`.
**Verdict:** Genuinely fixed. The annotations are substantive, not boilerplate.

### High: `thread_state_mut()` is fully unverified — VERIFIED FIXED

**Claim:** Added `AUDIT` annotation with call-site references.
**Evidence:** Confirmed at line 518: `AUDIT: Each call site must be manually reviewed to confirm the above invariants are preserved. See process/manager/mod.rs:1154,1169.` The existing trust boundary documentation (lines 496–516) was already excellent; the AUDIT tag adds traceability.
**Verdict:** Genuinely fixed. The annotation is concrete (references specific call sites), not a generic warning.

### Medium: `run()` omits raw context pointer — VERIFIED FIXED

**Claim:** Added `Modeling Note` documentation.
**Evidence:** Confirmed at lines 433–438: explains the `*mut ContextInformation` raw pointer is omitted because it's used for assembly-level context switching and cannot be meaningfully specified.
**Verdict:** Genuinely fixed. The note accurately describes the omission and its rationale.

### Medium: `join_cond()` omitted entirely — REJECTION VERIFIED JUSTIFIED

**Claim:** Rejected — requires ThreadState dependency changes.
**Verification:** `join_cond()` is consistently omitted across ALL thread modules (`ready.rs`, `interrupted.rs`, `state.rs`) because `Condvar` is an opaque sync boundary type (confirmed by `grep -rn join_cond verus/split/kernel/pm/thread/`). The `Condvar` identity modeling would require changes to the `ThreadState` dependency module first. This is a legitimate cross-module concern that should be addressed when `ThreadState` itself gains `Condvar` modeling.
**Verdict:** Rejection justified. This is a project-wide design decision, not a ReadyThread-specific gap.

### Medium: `wf()` does not include admission_time non-negativity — VERIFIED FIXED

**Claim:** Strengthened `wf()` to include `self.admission_time >= 0`; updated proof lemmas.
**Evidence:**
- `wf()` at `ready.spec.rs:133–134`: `self.state.wf() && self.admission_time >= 0` ✓
- `lemma_from_state_is_wf` at `ready.proof.rs:28–30`: now requires `state.wf()` AND `time >= 0` ✓
- `lemma_new_is_drop_safe` at `ready.proof.rs:78`: now requires `time >= 0` ✓
- Constructors `new()` and `from_state()` call `clock_now()` which ensures `result >= 0`, so existing callers are unaffected ✓
**Verdict:** Genuinely fixed. The spec change is consistent — preconditions tightened where needed, constructors already satisfy the new constraint.

### Low: Struct fields are public — ACKNOWLEDGED (No action needed)
### Low: Forwarding methods not in original source — ACKNOWLEDGED (No action needed)

### Low: EXIT_STATUS_INTERRUPTED hardcoded as 4 — VERIFIED FIXED

**Claim:** Added full conversion chain comment.
**Evidence:** Confirmed at `ready.spec.rs:70–72`: documents `ErrorCode::Interrupted (#[repr(i32)]) → From<ErrorCode> for i32 via errno as i32 → EINTR (errno.rs:21) = 4`.
**Verdict:** Genuinely fixed.

## New Issues Found

None significant. One minor observation:

### Low (New)

- **`lemma_new_then_run` does not require `time >= 0` despite claiming to exercise `new() + run()` composition**
  - Location: `ready.proof.rs:255–288`
  - Description: The doc comment states this lemma "exercises the composition of new() + run() specifications." However, `new()` always produces `admission_time >= 0` (via `clock_now()`), and `run()` requires `self.wf()` which now requires `admission_time >= 0`. The lemma accepts arbitrary `time` without constraint. This is logically sound — the lemma proves `ThreadState`-level properties (`post_state.wf()`, `post_state.spec_id()`) that don't depend on `admission_time`. But the claim of exercising "new() + run() composition" is slightly misleading since it bypasses the `ReadyThread::wf()` requirement that real `run()` imposes.
  - Impact: Documentation imprecision only. The lemma is correct and useful — it actually proves a *stronger* result (works for all `time`). No action required.

## Verification Results

Independently confirmed:
```
$ verus verus/split/lib.rs --crate-type=lib --verify-module kernel::pm::thread::ready
verification results:: 34 verified, 0 errors
```

Full crate also passes: 832 verified, 0 errors.

## Summary

All fixable issues from the previous review were genuinely addressed. The prover did not merely claim fixes — the changes are substantive and verifiable:

1. **CROSS-MODULE-CHECK annotations** (High → Fixed): Concrete, searchable, enumerate specific postconditions.
2. **AUDIT annotation** (High → Fixed): References specific call sites, not generic.
3. **Modeling Note for run()** (Medium → Fixed): Accurately documents the raw pointer omission.
4. **join_cond() rejection** (Medium → Justified): Consistent with project-wide Condvar omission across all thread modules.
5. **wf() strengthening** (Medium → Fixed): Correct spec change with consistent precondition updates.
6. **EXIT_STATUS_INTERRUPTED comment** (Low → Fixed): Full conversion chain documented.

The verification is sound: 34 conditions verified, 0 errors. The trust surface is minimal and well-documented. The one new minor issue (lemma documentation imprecision) does not affect correctness. The overall quality has improved from A- to A.
