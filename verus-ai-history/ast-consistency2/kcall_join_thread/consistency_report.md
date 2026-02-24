# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/join_thread.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/join_thread.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `join_thread` [join_thread.diff](join_thread.diff) | [join_thread_source.rs](join_thread_source.rs) | [join_thread_verus.rs](join_thread_verus.rs) | MISSING_IN_VERUS | 57-78 |  |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `join_thread` [join_thread.diff](join_thread.diff) | [join_thread_source.rs](join_thread_source.rs) | [join_thread_verus.rs](join_thread_verus.rs) | MISSING_IN_VERUS | ❌ |
