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
| `clear` [clear.diff](clear.diff) | [clear_source.rs](clear_source.rs) | [clear_verus.rs](clear_verus.rs) | MISMATCH | 26-28 | 237-257 |
| `has` [has.diff](has.diff) | [has_source.rs](has_source.rs) | [has_verus.rs](has_verus.rs) | MISMATCH | 30-32 | 268-279 |
| `set` [set.diff](set.diff) | [set_source.rs](set_source.rs) | [set_verus.rs](set_verus.rs) | MISMATCH | 22-24 | 210-230 |
| `default` [default_verus.rs](default_verus.rs) | EXTRA_IN_VERUS |  | 288-300 |
| `new` [new_verus.rs](new_verus.rs) | EXTRA_IN_VERUS |  | 191-203 |
| `to_mask` [to_mask_verus.rs](to_mask_verus.rs) | EXTRA_IN_VERUS |  | 169-184 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `clear` [clear.diff](clear.diff) | [clear_source.rs](clear_source.rs) | [clear_verus.rs](clear_verus.rs) | MISMATCH | ❌ |
| `has` [has.diff](has.diff) | [has_source.rs](has_source.rs) | [has_verus.rs](has_verus.rs) | MISMATCH | ❌ |
| `set` [set.diff](set.diff) | [set_source.rs](set_source.rs) | [set_verus.rs](set_verus.rs) | MISMATCH | ❌ |
| `default` [default_verus.rs](default_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new` [new_verus.rs](new_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `to_mask` [to_mask_verus.rs](to_mask_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Capabilities` [struct_Capabilities.diff](struct_Capabilities.diff) | [struct_Capabilities_source.rs](struct_Capabilities_source.rs) | [struct_Capabilities_verus.rs](struct_Capabilities_verus.rs): MISMATCH
