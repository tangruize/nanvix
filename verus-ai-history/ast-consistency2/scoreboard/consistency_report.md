# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/kcall/mod.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/kcall/mod.rs`

## Summary

- Functions matched: 0/6
- Functions mismatched: 0
- Missing in Verus: 6
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `dispatch` [dispatch_source.rs](dispatch_source.rs) | MISSING_IN_VERUS | 150-174 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 66-73 |  |
| `get_mut` [get_mut_source.rs](get_mut_source.rs) | MISSING_IN_VERUS | 105-115 |  |
| `handle` [handle_source.rs](handle_source.rs) | MISSING_IN_VERUS | 176-180 |  |
| `handled` [handled_source.rs](handled_source.rs) | MISSING_IN_VERUS | 182-185 |  |
| `init` [init_source.rs](init_source.rs) | MISSING_IN_VERUS | 192-195 |  |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `dispatch` [dispatch_source.rs](dispatch_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_mut` [get_mut_source.rs](get_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `handle` [handle_source.rs](handle_source.rs) | MISSING_IN_VERUS | ❌ |
| `handled` [handled_source.rs](handled_source.rs) | MISSING_IN_VERUS | ❌ |
| `init` [init_source.rs](init_source.rs) | MISSING_IN_VERUS | ❌ |

## Inconsistent Structs

- `KcallArgs` [struct_KcallArgs_source.rs](struct_KcallArgs_source.rs): MISSING_IN_VERUS
- `ScoreBoard` [struct_ScoreBoard_source.rs](struct_ScoreBoard_source.rs): MISSING_IN_VERUS
