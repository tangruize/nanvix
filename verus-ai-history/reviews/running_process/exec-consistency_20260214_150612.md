# Review: running_process Exec Consistency (claude-opus-4.6)

## Grade: A

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The consistency report identifies 13 MISMATCH functions and documents semantic equivalence for each. No code changes were made — all mismatches are structural adaptations required for Verus verification (type modeling, reference elision, oracle parameters). The documented justifications are individually examined below:

- **`new`**: Original takes `Box<ProcessState>`, `RunningThread`, `Option<NonEmptyVecDeque<T>>`. Verus takes `u64`, `Vec<u64>` plus count parameters. Body is struct construction from parameters — logically identical. ✅
- **`state` / `state_mut` / `running_mut`**: Original returns references (`&ProcessState`, `&mut ProcessState`, `&mut RunningThread`). Verus returns `u64` with `external_body`. Frame conditions (`self@ == old(self)@`) correctly model that accessing/borrowing should not change state. ✅
- **`get_tid`**: Original: `self.running.id()`. Verus: `self.running_thread_id`. The `.id()` accessor is modeled as direct field access. ✅
- **`schedule`**: Original pushes running thread onto ready queue via `NonEmptyVecDeque::push_back` or `NonEmptyVecDeque::new`. Verus uses `Vec::push`. Context pointer elided. Logic is identical: running→ready, return RunnableProcess. ✅
- **`sleep`**: Three branches match: (1) ready→RunnableProcess, (2) interrupted→InterruptedProcess.resume()→RunnableProcess, (3) SleepingProcess. The `alarm` parameter is correctly elided as it only affects SleepingThread timing, not state machine logic. Counter-based branching (`ready_count > 0`) is equivalent to `Option::is_some()`. ✅
- **`exit`**: Running→zombie, ready→zombie (via `map`+`terminate`), sleeping→interrupted (via `map`+`interrupt`). Verus models `map`+`terminate` as identity (thread IDs preserved) and `map`+`interrupt` as identity (sleeping→interrupted). The observation that `self.sleeping_threads.take()` at line 219 is always `None` is **correct** — sleeping threads were consumed at line 208. Verus faithfully passes `Vec::new()` for sleeping_thread_ids (line 981). ✅
- **`exit_thread`**: Four branches match original logic. The documented bug fix at line 286 (`self.zombie.take()` → `Some(zombie_threads)`) has been verified in the original source — the current source already has the fix. ✅
- **`wakeup`**: Oracle parameter `found` replaces `remove_if` search. Manual loop finds and removes tid. Precondition ties `found` to `spec_seq_contains`. ✅
- **`try_join_thread`**: Oracle parameter `tag` replaces multi-list search. Only zombie case mutates state. Precondition ties `tag` to `spec_try_join_thread`. ✅
- **`find_thread` / `find_thread_mut`**: Returns `Ghost<Option<int>>` delegating to `spec_find_thread()`. Documented as trust boundary due to Verus reference limitations. ✅

### 2. Were MISSING functions added with proper verification?

The report states 0 missing functions were added. All original functions are accounted for in the Verus code. The 3 extra functions (`vec_push_all`, `vec_remove_at`, `interrupted_resume`) and 5 extra structs (`RunnableProcess`, `SleepingProcess`, `InterruptedProcess`, `ZombieProcess`, `ScheduleResult`) plus result enums (`SleepResult`, `ExitResult`, `ExitThreadResult`) are properly justified as verification helpers and boundary models. ✅

### 3. Are equivalence justifications sound?

**Mostly sound, with minor observations:**

- **Type modeling** (`Box<ProcessState>` → `u64`, `RunningThread` → `u64`, etc.) is well-justified. The state machine logic depends only on thread identity (IDs), not thread content. PID immutability is a documented trust assumption discharged when ProcessState is verified.

- **Oracle parameters**: The approach is sound. Oracle preconditions (`found == spec_seq_contains(...)`, `tag == spec_try_join_thread(tid)`) shift search correctness to call sites. The critical caveat — "all callers must be verified (not external_body)" — is prominently documented.

- **`interrupted_resume()` external_body**: The contract is well-specified (PID preservation, front-thread-becomes-ready, tail becomes remaining interrupted, sleeping/zombie passthrough). The `inv()` postcondition ensures the result is well-formed. This is the strongest trust boundary; it models cross-module behavior. The contract is sufficiently detailed to constrain the behavior tightly.

- **`find_thread` / `find_thread_mut`**: These compute the result directly from the spec function (`Ghost(self@.find_thread(tid as int))`), making the exec code a trivial wrapper around the spec. While this is technically correct for verification, it means the exec code does not actually perform the linear scan — the search correctness is entirely a trust assumption. This is clearly documented but is the weakest point of the verification.

- **Counter fields**: The `ready_count`, `interrupted_count`, `sleeping_count`, `zombie_count` fields are tied to `Vec` lengths via `inv()`. This is a reasonable engineering choice for efficient branching without calling `.len()`.

### 4. Does the exec code now faithfully represent the original source?

**Yes, with documented deviations.** Function-by-function comparison confirms:

| Original Function | Verus Function | Faithful? |
|---|---|---|
| `new` | `new` | ✅ Same struct construction, different types |
| `state` | `state` | ✅ external_body, returns PID |
| `state_mut` | `state_mut` | ✅ external_body, frame condition |
| `running_mut` | `running_mut` | ✅ external_body, frame condition |
| `get_tid` | `get_tid` | ✅ Direct field access |
| `schedule` | `schedule` | ✅ Same logic: push running→ready |
| `sleep` | `sleep` | ✅ Three branches match |
| `exit` | `exit` | ✅ Same logic including sleeping_threads.take()=None observation |
| `exit_thread` | `exit_thread` | ✅ Four branches match, bug fix documented |
| `wakeup` | `wakeup` | ✅ Oracle replaces remove_if search |
| `try_join_thread` | `try_join_thread` | ✅ Oracle replaces multi-list search |
| `find_thread` | `find_thread` | ⚠️ Spec-only (trust boundary) |
| `find_thread_mut` | `find_thread_mut` | ⚠️ Spec-only (trust boundary) |

The `exit()` function's handling of `self.sleeping_threads.take()` at line 219 (original) deserves particular attention: the Verus model correctly identifies that this is always `None` because sleeping threads were consumed at line 208. The model passes `Vec::new()` (empty), which is faithful.

The `exit_thread()` bug fix claim is verified: the current original source at line 286 already passes `Some(zombie_threads)`, matching the Verus model.

### 5. Does verification still pass?

**Yes.** Verification output:
```
verification results:: 48 verified, 0 errors
```
- 4 `external_body` functions (all documented and justified).
- No `assume` or `admit` statements.
- Duration: 10 seconds.

## Issues Found

### Critical
- None.

### Minor

1. **`find_thread` / `find_thread_mut` are not truly verified at exec level.** These functions delegate entirely to the spec (`Ghost(self@.find_thread(...))`), meaning the exec code never performs the actual linear scan. The original source iterates through all thread lists; the Verus version trusts the spec to be correct. This is documented as a limitation, but it means any bug in the spec's search order or predicate logic would not be caught by exec-level verification. This is acceptable given current Verus limitations on reference-returning functions but should be revisited when Verus adds that support.

2. **`state_mut()` frame condition may be too strong.** The `ensures self@ == old(self)@` on `state_mut()` asserts that calling `state_mut()` never changes any modeled field. In the original, `state_mut()` returns `&mut ProcessState`, which *could* be used to mutate the PID. The frame condition assumes callers won't do this. This is a reasonable trust assumption but should be flagged for downstream callers that might violate it.

3. **`wakeup()` precondition `ready_count < u64::MAX`** is sound but not present in the original. Real systems will never approach 2^64 threads, so this is purely a verification artifact for overflow prevention.

4. **`exit()` zombie ordering**: The Verus model produces `zombie.push(running).add(ready)`, which means the running thread's zombie entry is at position `old_zombie.len()` and ready thread zombies follow. The original does the same (`push_back` running, then `append` ready zombies). Ordering matches. ✅

## Summary

The exec consistency fix is well-executed. All 13 MISMATCH functions have thorough semantic equivalence documentation, and no functions are missing. The verification model is a faithful abstraction of the original `RunningProcess`, with type simplifications (kernel types → `u64`/`Vec<u64>`) that preserve the state machine logic. The 4 `external_body` functions are properly justified trust boundaries, and the oracle parameter pattern for `wakeup` and `try_join_thread` is sound with clearly documented preconditions. The discovered bug in `exit_thread()` (zombie thread loss) has been confirmed as fixed in the original source.

The main limitation is `find_thread` / `find_thread_mut`, which are spec-only wrappers rather than verified exec implementations. This is an acknowledged Verus limitation, not a modeling error.

Verification passes cleanly: 48 verified, 0 errors, no cheating (assume/admit).
