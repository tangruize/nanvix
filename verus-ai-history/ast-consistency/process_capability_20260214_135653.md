# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/capability.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/capability.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 3
- Missing in Verus: 0
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `clear` | MISMATCH | 26-28 | 237-257 |
| `has` | MISMATCH | 30-32 | 268-279 |
| `set` | MISMATCH | 22-24 | 210-230 |
| `default` | EXTRA_IN_VERUS |  | 288-300 |
| `new` | EXTRA_IN_VERUS |  | 191-203 |
| `to_mask` | EXTRA_IN_VERUS |  | 169-184 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `clear` | MISMATCH | ❌ |
| `has` | MISMATCH | ❌ |
| `set` | MISMATCH | ❌ |
| `default` | EXTRA_IN_VERUS | ❌ |
| `new` | EXTRA_IN_VERUS | ❌ |
| `to_mask` | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Capabilities`: MISMATCH
