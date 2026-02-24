# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/interrupted.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/interrupted.rs`

## Summary

- Functions matched: 0/8
- Functions mismatched: 8
- Missing in Verus: 0
- Extra in Verus: 1
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `find_thread` [find_thread.diff](find_thread.diff) | [find_thread_source.rs](find_thread_source.rs) | [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | 110-135 | 510-520 |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) | [find_thread_mut_source.rs](find_thread_mut_source.rs) | [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | 151-179 | 535-551 |
| `from_sleeping` [from_sleeping.diff](from_sleeping.diff) | [from_sleeping_source.rs](from_sleeping_source.rs) | [from_sleeping_verus.rs](from_sleeping_verus.rs) | MISMATCH | 60-72 | 210-240 |
| `interrupt` [interrupt.diff](interrupt.diff) | [interrupt_source.rs](interrupt_source.rs) | [interrupt_verus.rs](interrupt_verus.rs) | MISMATCH | 182-184 | 572-578 |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 47-58 | 167-193 |
| `resume` [resume.diff](resume.diff) | [resume_source.rs](resume_source.rs) | [resume_verus.rs](resume_verus.rs) | MISMATCH | 82-94 | 317-436 |
| `state` [state.diff](state.diff) | [state_source.rs](state_source.rs) | [state_verus.rs](state_verus.rs) | MISMATCH | 74-76 | 249-256 |
| `state_mut` [state_mut.diff](state_mut.diff) | [state_mut_source.rs](state_mut_source.rs) | [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | 78-80 | 268-283 |
| `resume_with_valid_clock` [resume_with_valid_clock_verus.rs](resume_with_valid_clock_verus.rs) | EXTRA_IN_VERUS |  | 458-480 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `find_thread` [find_thread.diff](find_thread.diff) | [find_thread_source.rs](find_thread_source.rs) | [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | ❌ |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) | [find_thread_mut_source.rs](find_thread_mut_source.rs) | [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | ❌ |
| `from_sleeping` [from_sleeping.diff](from_sleeping.diff) | [from_sleeping_source.rs](from_sleeping_source.rs) | [from_sleeping_verus.rs](from_sleeping_verus.rs) | MISMATCH | ❌ |
| `interrupt` [interrupt.diff](interrupt.diff) | [interrupt_source.rs](interrupt_source.rs) | [interrupt_verus.rs](interrupt_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `resume` [resume.diff](resume.diff) | [resume_source.rs](resume_source.rs) | [resume_verus.rs](resume_verus.rs) | MISMATCH | ❌ |
| `state` [state.diff](state.diff) | [state_source.rs](state_source.rs) | [state_verus.rs](state_verus.rs) | MISMATCH | ❌ |
| `state_mut` [state_mut.diff](state_mut.diff) | [state_mut_source.rs](state_mut_source.rs) | [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | ❌ |
| `resume_with_valid_clock` [resume_with_valid_clock_verus.rs](resume_with_valid_clock_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `InterruptedProcess` [struct_InterruptedProcess.diff](struct_InterruptedProcess.diff) | [struct_InterruptedProcess_source.rs](struct_InterruptedProcess_source.rs) | [struct_InterruptedProcess_verus.rs](struct_InterruptedProcess_verus.rs): MISMATCH
- `RunnableProcess` [struct_RunnableProcess_verus.rs](struct_RunnableProcess_verus.rs): EXTRA_IN_VERUS
