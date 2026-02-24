# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/interrupted.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/interrupted.rs`

## Summary

- Functions matched: 1/6
- Functions mismatched: 4
- Missing in Verus: 1
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | MISMATCH | 61-63 | 141-152 |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | MISMATCH | 74-76 | 159-166 |
| `join_cond` [join_cond.diff](join_cond.diff) [join_cond_source.rs](join_cond_source.rs) [join_cond_verus.rs](join_cond_verus.rs) | MISSING_IN_VERUS | 127-129 |  |
| `resume` [resume.diff](resume.diff) [resume_source.rs](resume_source.rs) [resume_verus.rs](resume_verus.rs) | MISMATCH | 113-116 | 195-208 |
| `thread_state` [thread_state.diff](thread_state.diff) [thread_state_source.rs](thread_state_source.rs) [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | 87-89 | 174-182 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | MISMATCH | ❌ |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | MISMATCH | ❌ |
| `join_cond` [join_cond.diff](join_cond.diff) [join_cond_source.rs](join_cond_source.rs) [join_cond_verus.rs](join_cond_verus.rs) | MISSING_IN_VERUS | ❌ |
| `resume` [resume.diff](resume.diff) [resume_source.rs](resume_source.rs) [resume_verus.rs](resume_verus.rs) | MISMATCH | ❌ |
| `thread_state` [thread_state.diff](thread_state.diff) [thread_state_source.rs](thread_state_source.rs) [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | ❌ |
| `thread_state_mut` | MATCH | ✅ |

## Inconsistent Structs

- `InterruptedThread` [struct_InterruptedThread.diff](struct_InterruptedThread.diff) [struct_InterruptedThread_source.rs](struct_InterruptedThread_source.rs) [struct_InterruptedThread_verus.rs](struct_InterruptedThread_verus.rs): MISMATCH
- `ReadyThread` [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs): EXTRA_IN_VERUS
