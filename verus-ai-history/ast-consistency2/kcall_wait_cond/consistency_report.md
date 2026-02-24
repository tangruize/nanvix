# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/wait_cond.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/wait_cond.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `wait_cond` [wait_cond.diff](wait_cond.diff) [wait_cond_source.rs](wait_cond_source.rs) [wait_cond_verus.rs](wait_cond_verus.rs) | MISSING_IN_VERUS | 72-138 |  |
| `cond_wait_model` [cond_wait_model_verus.rs](cond_wait_model_verus.rs) | EXTRA_IN_VERUS |  | 412-424 |
| `get_cond_and_wait_model` [get_cond_and_wait_model_verus.rs](get_cond_and_wait_model_verus.rs) | EXTRA_IN_VERUS |  | 536-863 |
| `mutex_lock_model` [mutex_lock_model_verus.rs](mutex_lock_model_verus.rs) | EXTRA_IN_VERUS |  | 463-471 |
| `parse_timeout_model` [parse_timeout_model_verus.rs](parse_timeout_model_verus.rs) | EXTRA_IN_VERUS |  | 504-517 |
| `wait_cond_model` [wait_cond_model_verus.rs](wait_cond_model_verus.rs) | EXTRA_IN_VERUS |  | 619-861 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `wait_cond` [wait_cond.diff](wait_cond.diff) [wait_cond_source.rs](wait_cond_source.rs) [wait_cond_verus.rs](wait_cond_verus.rs) | MISSING_IN_VERUS | ❌ |
| `cond_wait_model` [cond_wait_model_verus.rs](cond_wait_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_cond_and_wait_model` [get_cond_and_wait_model_verus.rs](get_cond_and_wait_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `mutex_lock_model` [mutex_lock_model_verus.rs](mutex_lock_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `parse_timeout_model` [parse_timeout_model_verus.rs](parse_timeout_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `wait_cond_model` [wait_cond_model_verus.rs](wait_cond_model_verus.rs) | EXTRA_IN_VERUS | ❌ |
