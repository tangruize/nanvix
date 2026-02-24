# Exec Consistency Fix: runnable

## Summary
- Mismatches fixed: 0 (all 4 documented as verification-necessary equivalences)
- Missing functions added: 2 (`earliest_admission_time`, `state`)
- Documented equivalences: 4 (`new`, `from_state`, `run`, `terminate`)
- Missing functions documented as intentionally omitted: 3 (`state_mut`, `find_thread`, `find_thread_mut`)
- Extra functions documented as justified: 5 (`clock_now`, `pid_i32`, `vec_concat`, `vec_remove_at`, `vec_search`)
- Extra structs documented as justified: 3 (`InterruptedProcess`, `RunningProcess`, `ZombieProcess`)
- AST tool false positives resolved: 2 (`add_thread`, `wakeup` exist but have abstracted signatures)

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | Documented equivalence | Type abstraction for verification: `ReadyThread` → `(i64, i64)` (tid, time), `Vmem` omitted (HAL boundary). Logic identical: creates process with one ready thread, empty other lists. |
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | Documented equivalence | Type abstraction: `Box<ProcessState>` → `ProcessIdentifier`, `NonEmptyVecDeque<T>` → `Vec<i64>`, `Option<NonEmptyVecDeque<T>>` → `Vec<i64>`. Added `interrupted_count`/`sleeping_count` exec counters for branch decisions in `terminate()`. Logic identical: assigns all fields. |
| `run` [run.diff](run.diff) [run_source.rs](run_source.rs) [run_verus.rs](run_verus.rs) | Documented equivalence | Same algorithm: linear scan for earliest admission time, remove selected thread, create RunningProcess. `ContextInformation`, `VirtualAddress` omitted (HAL boundary). `InterruptReason` modeled as `i64`. The `unreachable!()` branch is absent because `wf()` guarantees non-empty list. |
| `terminate` [terminate.diff](terminate.diff) [terminate_source.rs](terminate_source.rs) [terminate_verus.rs](terminate_verus.rs) | Documented equivalence | Same logic flow: ready→zombie, sleeping→interrupted, branch on existence of interrupted/sleeping. Uses exec-level counters (`interrupted_count`, `sleeping_count`) instead of `Option::take()` pattern matching; `wf()` ties counters to vector lengths. Returns `TerminateResult` enum instead of `Result<InterruptedProcess, ZombieProcess>`. |
| `state` [state_source.rs](state_source.rs) | Added exec function | Returns `ProcessIdentifier` (copy). Models `state().pid` since `ProcessState` is abstracted to PID. The original returns `&ProcessState`; Verus cannot express borrow-returning accessors. |
| `state_mut` [state_mut_source.rs](state_mut_source.rs) | Documented omission | Returns `&mut ProcessState`. Verus cannot express mutable references to abstracted types. Cross-module obligation: callers must preserve PID immutability. Documented in spec file. |
| `earliest_admission_time` [earliest_admission_time_source.rs](earliest_admission_time_source.rs) | Added exec function | Computes minimum admission time via linear scan. Omits the `unwrap_or(clock::now())` fallback from the original, which is dead code given the `NonEmptyVecDeque` invariant. Proven to equal `spec_earliest_admission_time()`. |
| `find_thread` [find_thread_source.rs](find_thread_source.rs) | Documented omission | Returns `Option<ThreadRef<'_>>` containing references into internal collections. Verus cannot express reference return types. Modeled spec-only via `spec_find_thread()`. |
| `find_thread_mut` [find_thread_mut_source.rs](find_thread_mut_source.rs) | Documented omission | Returns `Option<ThreadRefMut<'_>>`. Same Verus limitation as `find_thread`. Documented in spec file. |
| `add_thread` [add_thread_source.rs](add_thread_source.rs) | AST false positive | Already exists at line 879 with abstracted signature: `(self, ready_tid: i64, ready_time: i64)` instead of `(mut self, ready_thread: ReadyThread)`. Logic is identical: pushes thread to ready list. |
| `wakeup` [wakeup_source.rs](wakeup_source.rs) | AST false positive | Already exists at line 645 with abstracted signature: `(self, tid: i64)` instead of `(mut self, tid: ThreadIdentifier)`. Logic is identical: search sleeping list, move to ready queue. |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | Justified extra | External boundary model of `clock::now()`. Required for `wakeup()` and `earliest_admission_time()` to assign admission times. Minimal postcondition (result ≥ 0). |
| `pid_i32` [pid_i32_verus.rs](pid_i32_verus.rs) | Justified extra | Helper for PID access. Models `state().pid.into()`. Provides exec-level PID retrieval since `state()` returns an abstracted type. |
| `vec_concat` [vec_concat_verus.rs](vec_concat_verus.rs) | Justified extra | Verification helper for `terminate()`. Verus cannot verify `Vec::extend()`; explicit loop with invariants required. |
| `vec_remove_at` [vec_remove_at_verus.rs](vec_remove_at_verus.rs) | Justified extra | Verification helper for `run()` and `wakeup()`. Builds a new Vec excluding one element, with proven spec equivalence to `spec_remove_at()`. |
| `vec_search` [vec_search_verus.rs](vec_search_verus.rs) | Justified extra | Verification helper for `wakeup()`. Linear search with proven postcondition, replacing the `remove_if()` closure from the original. |
| `InterruptedProcess` [struct_InterruptedProcess_verus.rs](struct_InterruptedProcess_verus.rs) | Justified extra struct | Boundary model of sibling module's type. Required as return type for `terminate()`. |
| `RunningProcess` [struct_RunningProcess_verus.rs](struct_RunningProcess_verus.rs) | Justified extra struct | Boundary model of sibling module's type. Required as return type for `run()`. |
| `ZombieProcess` [struct_ZombieProcess_verus.rs](struct_ZombieProcess_verus.rs) | Justified extra struct | Boundary model of sibling module's type. Required as return type for `terminate()`. |
| `RunnableProcess` [struct_RunnableProcess.diff](struct_RunnableProcess.diff) [struct_RunnableProcess_source.rs](struct_RunnableProcess_source.rs) [struct_RunnableProcess_verus.rs](struct_RunnableProcess_verus.rs) | Documented struct mismatch | Fields use abstract types (`Vec<i64>` instead of `NonEmptyVecDeque<ReadyThread>`, etc.) and include exec-level counters (`interrupted_count`, `sleeping_count`) for branch decisions without oracle parameters. |

## Verification: PASS

```
verification results:: 70 verified, 0 errors
Duration: 10s
```

Verification improved from 67 to 70 verified items after adding `earliest_admission_time` and `state`.
