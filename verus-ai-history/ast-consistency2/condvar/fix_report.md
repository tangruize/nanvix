# Exec Consistency Fix: condvar

## Summary
- Mismatches fixed: 1 (new — documented as equivalent)
- Missing functions added: 6 (notify_first, notify_process, notify_thread, notify_all, wait, drop_check)
- Documented equivalences: 4 (new, Condvar struct, CondvarInner, reference_count/fmt)

## Context

The Verus condvar is a **verification model**, not a runtime replacement (documented in file header).
It uses `Vec<(i32, i32)>` + `len: usize` instead of `Arc<RefCell<LinkedList<(ProcessIdentifier, ThreadIdentifier)>>>`.
The "extra" functions are decomposed verification helpers; the "missing" functions are original API
names that map to these helpers. All extra functions are justified and documented in the API Mapping table.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | DOCUMENTED | Structurally different (Vec vs Arc\<RefCell\<LinkedList\>\>) but semantically equivalent: both create an empty sleeping queue. The different representation is necessary for verification (Verus cannot reason about Arc/RefCell/LinkedList). |
| `Condvar` struct | DOCUMENTED | Original has `inner: Arc<CondvarInner>`, Verus has `len: usize, sleeping: Vec<(i32, i32)>`. Intentional flattening for verification model. |
| `CondvarInner` struct | DOCUMENTED | Flattened into `Condvar` in the verification model. The original's `RefCell<LinkedList<...>>` interior mutability pattern is replaced by `&mut self` methods. |
| `notify_first` [notify_first.diff](notify_first.diff) [notify_first_source.rs](notify_first_source.rs) [notify_first_verus.rs](notify_first_verus.rs) | ADDED | Wrapper around `dequeue_first()`. Returns `u32` (0 or 1) instead of `Result<u32, Error>` since `ProcessManager::wakeup()` is not modeled. |
| `notify_process` [notify_process.diff](notify_process.diff) [notify_process_source.rs](notify_process_source.rs) [notify_process_verus.rs](notify_process_verus.rs) | ADDED | Wrapper around `try_remove_by_pid()`. Parameters externalize the `position()` search result (`has_match`, `match_idx`) since the search depends on runtime queue state. |
| `notify_thread` [notify_thread.diff](notify_thread.diff) [notify_thread_source.rs](notify_thread_source.rs) [notify_thread_verus.rs](notify_thread_verus.rs) | ADDED | Wrapper around `try_remove_by_tid()`. Same parameter externalization as `notify_process`. |
| `notify_all` [notify_all.diff](notify_all.diff) [notify_all_source.rs](notify_all_source.rs) [notify_all_verus.rs](notify_all_verus.rs) | ADDED | Wrapper around `clear()`. Returns `usize` (total entries) instead of `Result<u32, Error>` since wakeup success/failure is not modeled. |
| `wait` [wait.diff](wait.diff) [wait_source.rs](wait_source.rs) [wait_verus.rs](wait_verus.rs) | ADDED | Wrapper around `try_enqueue()`. Models queue insertion + alarm check. `ProcessManager::sleep()` and error cleanup (`retain()`) are modeled separately by `remove_entry()` and proven correct by `lemma_wait_cleanup_restores_state`. |
| `drop_check` [drop_check_verus.rs](drop_check_verus.rs) | ADDED | Models `CondvarInner::Drop::drop()` panic check. Precondition `spec_drop_safe()` replaces the runtime panic with a static verification check. |
| `reference_count` [reference_count_source.rs](reference_count_source.rs) | DOCUMENTED | Arc-specific (`Arc::strong_count`), not relevant to queue management. See trust assumption T6. Cannot model without Arc. |
| `fmt` [fmt_source.rs](fmt_source.rs) | DOCUMENTED | Debug trait implementation for formatting. Trait impls are not modeled in the Verus verification. |
| `clear` [clear_verus.rs](clear_verus.rs) | KEPT | Justified verification helper modeling `notify_all()` queue drain. Documented in API Mapping. |
| `dequeue_first` [dequeue_first_verus.rs](dequeue_first_verus.rs) | KEPT | Justified verification helper modeling `notify_first()` queue removal. Documented in API Mapping. |
| `enqueue` [enqueue_verus.rs](enqueue_verus.rs) | KEPT | Justified verification helper modeling `wait()` queue insertion. Documented in API Mapping. |
| `try_enqueue` [try_enqueue_verus.rs](try_enqueue_verus.rs) | KEPT | Justified verification helper modeling `wait(alarm)` conditional insertion. Documented in API Mapping. |
| `remove_at` [remove_at_verus.rs](remove_at_verus.rs) | KEPT | Core removal primitive used by `remove_entry`, `remove_by_pid`, `remove_by_tid`. |
| `remove_entry` [remove_entry_verus.rs](remove_entry_verus.rs) | KEPT | Models `wait()` failure cleanup (`retain()`). Documented in API Mapping. |
| `remove_by_pid` [remove_by_pid_verus.rs](remove_by_pid_verus.rs) | KEPT | Models `notify_process()` removal with first-match semantics. |
| `remove_by_tid` [remove_by_tid_verus.rs](remove_by_tid_verus.rs) | KEPT | Models `notify_thread()` removal with first-match semantics. |
| `try_remove_by_pid` [try_remove_by_pid_verus.rs](try_remove_by_pid_verus.rs) | KEPT | Models `notify_process()` including not-found case. |
| `try_remove_by_tid` [try_remove_by_tid_verus.rs](try_remove_by_tid_verus.rs) | KEPT | Models `notify_thread()` including not-found case. |
| `is_empty` [is_empty_verus.rs](is_empty_verus.rs) | KEPT | Queue state observer. |
| `get_len` [get_len_verus.rs](get_len_verus.rs) | KEPT | Queue state observer. |

## Verification: PASS

```
verification results:: 59 verified, 0 errors
Duration: 9s
```
