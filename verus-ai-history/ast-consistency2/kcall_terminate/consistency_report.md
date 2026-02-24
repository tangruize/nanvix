# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/terminate.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/terminate.rs`

## Summary

- Functions matched: 0/1
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `terminate` [terminate.diff](terminate.diff) [terminate_source.rs](terminate_source.rs) [terminate_verus.rs](terminate_verus.rs) | MISSING_IN_VERUS | 21-34 |  |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `terminate` [terminate.diff](terminate.diff) [terminate_source.rs](terminate_source.rs) [terminate_verus.rs](terminate_verus.rs) | MISSING_IN_VERUS | ❌ |
