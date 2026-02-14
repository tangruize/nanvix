# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/signal_cond.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/signal_cond.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `signal_cond` | MISSING_IN_VERUS | 50-75 |  |
| `drop_cond_model` | EXTRA_IN_VERUS |  | 356-364 |
| `put_cond_model` | EXTRA_IN_VERUS |  | 384-551 |
| `signal_cond_model` | EXTRA_IN_VERUS |  | 426-549 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `signal_cond` | MISSING_IN_VERUS | ❌ |
| `drop_cond_model` | EXTRA_IN_VERUS | ❌ |
| `put_cond_model` | EXTRA_IN_VERUS | ❌ |
| `signal_cond_model` | EXTRA_IN_VERUS | ❌ |
