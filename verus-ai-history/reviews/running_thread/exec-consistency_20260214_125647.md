# Review: running_thread Exec Consistency (claude-opus-4.6)

## Grade: A

## Files Reviewed

- Original: `src/kernel/src/pm/thread/running.rs`
- Exec: `verus/split/kernel/pm/thread/running.rs`
- Spec: `verus/split/kernel/pm/thread/running.spec.rs`
- Proof: `verus/split/kernel/pm/thread/running.proof.rs`
- Consistency report: `verus-ai-history/ast-consistency/running_thread_20260214_125647_fix.md`

## Verification Result

```
47 verified, 0 errors
```

No `assume()` calls. No `external_body` annotations. One `#[verifier::external]` on
`thread_state_mut` (justified — Verus cannot express `&mut T` return types).

## Function-by-Function Analysis

### MISMATCH Functions (8 documented equivalences)

| # | Function | Verdict | Notes |
|---|----------|---------|-------|
| 1 | `from_state` | ✅ Sound | `Box<ThreadState>` → `ThreadState` (transparent). `Self { state }` ≡ `RunningThread { state: state }`. `pub(super)` → `pub` for proof access. |
| 2 | `sleep` | ✅ Sound | Core delegation `SleepingThread::from_state(self.state, alarm)` identical. `*mut ContextInformation` return omitted (HAL boundary, unsafe). `Option<SystemTime>` → `Option<int>`. `mut self` → `self` (no longer calls `context_mut()`). |
| 3 | `schedule` | ✅ Sound | Same pattern as `sleep`. Core delegation `ReadyThread::from_state(self.state)` identical. `*mut ContextInformation` omitted. |
| 4 | `id` | ✅ Identical | Body: `self.state.id()` — exact match. |
| 5 | `thread_state` | ✅ Identical | Body: `&self.state` — exact match. |
| 6 | `exit` | ✅ Sound | Core delegation `ZombieThread::from_state(self.state, status)` identical. `ExitStatus` → `int`. `*mut ContextInformation` omitted. |
| 7 | `put_mutex_guard` | ✅ Sound | `MutexAddress` → `u64` (widening). `MutexGuard` elided (RAII payload, protocol-only). Delegation `self.state.store_mutex_guard(address)` identical. Added preconditions (`!spec_has_mutex`, `count < MAX`) are strengthenings that correctly model non-recursive mutex semantics. |
| 8 | `take_mutex_guard` | ✅ Sound | `MutexAddress` → `u64`. `Option<MutexGuard>` return → unit (precondition `spec_has_mutex` makes `None` unreachable — trust assumption T2). Delegation identical. |

### MISSING Functions (1)

| # | Function | Verdict | Notes |
|---|----------|---------|-------|
| 1 | `join_cond` | ✅ Justified omission | Returns `Condvar` (sync boundary type entirely elided from the `ThreadState` model). The omission is well-documented in the module header and spec header under "Trust Boundary". Adding it would require restructuring the `ThreadState` model to include sync primitives. |

### EXTRA Structures (3 boundary models)

| # | Struct | Verdict | Notes |
|---|--------|---------|-------|
| 1 | `SleepingThread` | ✅ Justified | Boundary model of sibling module. Needed to type-check `sleep()` return. Cross-module verification obligations documented. |
| 2 | `ReadyThread` | ✅ Justified | Boundary model for `schedule()`. `admission_time` field intentionally omitted (scheduling property). |
| 3 | `ZombieThread` | ✅ Justified | Boundary model for `exit()`. Cross-module verification obligations documented. |

### Unverified Function (1)

| # | Function | Verdict | Notes |
|---|----------|---------|-------|
| 1 | `thread_state_mut` | ✅ Properly handled | Marked `#[verifier::external]` because Verus cannot express `&mut T` return types. Body `&mut self.state` is identical to original. Trust obligations (preserve `wf()` and `spec_id()`) are clearly documented as intended postconditions. |

## Issues Found

### Critical

- None.

### Minor

1. **`put_mutex_guard` precondition strengthening**: The precondition `!spec_has_mutex(address@)` is strictly stronger than the original, which silently overwrites via `BTreeMap::insert`. This is documented as modeling non-recursive mutex semantics and is a reasonable strengthening, but callers at the verification boundary must be aware that double-lock is rejected by the model even though the original code would succeed (with data loss). This is correctly documented in the function's doc comment.

2. **`take_mutex_guard` return type elision**: The original returns `Option<MutexGuard>`, but the verified version returns unit. The precondition makes `None` unreachable, so this is a valid API strengthening. However, any caller outside the verification boundary that relied on the `None` path for defensive error handling would need to ensure the precondition holds. This is correctly documented as trust assumption T2.

3. **`ReadyThread` boundary model omits `admission_time`**: The real `ReadyThread::from_state` sets `admission_time = clock::now()`. This is documented as intentionally omitted (scheduling property). When `ReadyThread` gets its own verification module, the boundary model's postconditions should be cross-checked. This obligation is documented in the code.

## Spec & Proof Quality

- **Spec file**: Clean separation of View types and spec functions. All four types (`RunningThread`, `SleepingThread`, `ReadyThread`, `ZombieThread`) have well-formedness predicates (`closed spec fn wf`), View impls, and comprehensive spec accessors (`spec_id`, `spec_has_mutex`, `spec_drop_safe`, etc.).

- **Proof file**: 47 lemmas covering construction, identity preservation, state transitions (sleep/schedule/exit), mutex accounting, drop safety, view equality, and composite properties (e.g., `lemma_from_state_then_schedule`, `lemma_acquire_then_release_restores_mutex_state`). All machine-checked with no `assume()`.

- **Trust boundary documentation**: Thorough. Module header documents all type abstractions, HAL boundary omissions, and trust assumptions. Individual functions document their specific modeling notes and cross-module verification obligations.

## Summary

The exec consistency fix is well-executed. All 8 MISMATCH functions faithfully preserve the original executable logic with clearly documented type abstractions (`Box` removal, `SystemTime` → `int`, `ExitStatus` → `int`, `MutexAddress` → `u64`, `*mut ContextInformation` omission). The 1 MISSING function (`join_cond`) is properly justified as a sync boundary omission. The 3 EXTRA boundary models are necessary and include cross-module verification obligations. The code passes verification with 47 verified, 0 errors, no `assume()` calls, and only one `#[verifier::external]` that is properly justified. The two API strengthenings (`put_mutex_guard` no-double-lock precondition and `take_mutex_guard` return type elision) are reasonable modeling choices with documented trust assumptions. Grade A — no critical issues; minor observations are all properly documented in the code itself.
