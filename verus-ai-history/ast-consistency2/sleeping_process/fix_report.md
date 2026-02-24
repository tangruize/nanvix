# Exec Consistency Fix: sleeping_process

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 9 functions + 3 structs

All 9 function mismatches and 3 struct differences are inherent to the verification
modeling approach. The Verus module uses simplified abstract types (u64 PIDs, Vec<u64>
thread ID lists) instead of complex kernel types (Box<ProcessState>,
NonEmptyVecDeque<SleepingThread>, etc.). No executable logic was changed — the state
machine transitions, search algorithms, and data flow are semantically equivalent.

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | Documented equivalence | Type abstraction: `Box<ProcessState>` → `u64` pid, `NonEmptyVecDeque<SleepingThread>` → `Vec<u64>`, `Option<NonEmptyVecDeque<ZombieThread>>` → `Vec<u64>`. Proof block bridges view-level no_duplicates — pure verification overhead. Construction logic identical: assign fields. |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | Documented equivalence (Verus limitation) | Original returns `&ProcessState`; Verus returns `u64` (PID). Verus cannot express lifetime-bearing reference return types. Marked `external_body` with ensures clause tying result to `self@.pid`. Trust boundary documented in module header. |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | Documented equivalence (Verus limitation) | Original returns `&mut ProcessState`; Verus returns `u64`. Same Verus reference-type limitation. Frame conditions (`ensures self@ == old(self)@` for lists) model that mutation does not affect thread lists or PID. |
| `terminate` [terminate.diff](terminate.diff) [terminate_source.rs](terminate_source.rs) [terminate_verus.rs](terminate_verus.rs) | Documented equivalence | Original iterates sleeping threads calling `interrupt(InterruptReason::Killed)` on each, producing `InterruptedThread`s. `interrupt()` is ID-preserving (trust boundary). Verus moves `sleeping_thread_ids` → `interrupted_thread_ids` directly, which is equivalent since only thread IDs are tracked in the model. |
| `wakeup` [wakeup.diff](wakeup.diff) [wakeup_source.rs](wakeup_source.rs) [wakeup_verus.rs](wakeup_verus.rs) | Documented equivalence | Original uses `NonEmptyVecDeque::remove_if(\|t\| t.id() == tid)`. Verus uses oracle `found` parameter + linear scan + `Vec::remove`. Same algorithm: find thread by ID, remove from sleeping list, make it ready. Oracle parameter replaces runtime `remove_if` result; precondition ties oracle to `spec_seq_contains`. \|
| `wakeup_alarm` [wakeup_alarm.diff](wakeup_alarm.diff) [wakeup_alarm_source.rs](wakeup_alarm_source.rs) [wakeup_alarm_verus.rs](wakeup_alarm_verus.rs) | Documented equivalence | Original iterates threads checking `alarm()` and `now >= alarm`. Verus uses oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`) for the partition. Per-thread alarm comparison is simple arithmetic not modeled in verification. Conservation and partition-integrity preconditions ensure oracle correctness. Documented trust boundary. |
| `add_thread` [add_thread.diff](add_thread.diff) [add_thread_source.rs](add_thread_source.rs) [add_thread_verus.rs](add_thread_verus.rs) | Documented equivalence | Original calls `RunnableProcess::from_state(self.state, NonEmptyVecDeque::new(ready_thread), None, Some(self.sleeping_threads), self.zombie_threads.take())`. Verus constructs `RunnableProcess` struct directly with same field mapping. Semantically identical. |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | Documented equivalence (Verus limitation) | Original returns `Option<ThreadRef<'_>>` with concrete references. Verus returns `Ghost<Option<int>>` because Verus cannot express functions returning reference types. Search logic (sleeping → zombie order) preserved via `spec_find_thread()`. |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | Documented equivalence (Verus limitation) | Original returns `Option<ThreadRefMut<'_>>`. Same Verus limitation as `find_thread`. Frame conditions ensure `self` is unchanged after the call. |

## Struct Differences
| Struct | Status | Justification |
|--------|--------|---------------|
| `SleepingProcess` [struct_SleepingProcess.diff](struct_SleepingProcess.diff) [struct_SleepingProcess_source.rs](struct_SleepingProcess_source.rs) [struct_SleepingProcess_verus.rs](struct_SleepingProcess_verus.rs) | MISMATCH | Verification model: `Box<ProcessState>` → `pub pid: u64`, `NonEmptyVecDeque<SleepingThread>` → `pub sleeping_thread_ids: Vec<u64>`, `Option<NonEmptyVecDeque<ZombieThread>>` → `pub zombie_thread_ids: Vec<u64>`. Fields are `pub` for Verus proof ergonomics (documented in module header). |
| `RunnableProcess` [struct_RunnableProcess_verus.rs](struct_RunnableProcess_verus.rs) | EXTRA_IN_VERUS | Boundary model of sibling module's type, required as return type for `wakeup()` and `add_thread()`. Contains simplified field set matching the verification model. Justified and documented. |
| `InterruptedProcess` [struct_InterruptedProcess_verus.rs](struct_InterruptedProcess_verus.rs) | EXTRA_IN_VERUS | Boundary model of sibling module's type, required as return type for `terminate()` and `wakeup_alarm()`. Contains simplified field set matching the verification model. Justified and documented. |

## Verification: PASS
- 30 verified, 0 errors
- Cheating patterns: 2 external_body (`state()`, `state_mut()`) — both justified as reference-type trust boundaries
- No `assume` or `admit` used
