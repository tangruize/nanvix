# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/kstack.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/kstack.rs`

## Summary

- Functions matched: 0/6
- Functions mismatched: 4
- Missing in Verus: 2
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | MISMATCH | 90-92 | 290-298 |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 135-140 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 123-131 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 57-62 | 193-256 |
| `size` [size.diff](size.diff) | [size_source.rs](size_source.rs) | [size_verus.rs](size_verus.rs) | MISMATCH | 73-75 | 266-276 |
| `top` [top.diff](top.diff) | [top_source.rs](top_source.rs) | [top_verus.rs](top_verus.rs) | MISMATCH | 107-115 | 313-323 |
| `contains` [contains_verus.rs](contains_verus.rs) | EXTRA_IN_VERUS |  | 356-363 |
| `has_room` [has_room_verus.rs](has_room_verus.rs) | EXTRA_IN_VERUS |  | 427-438 |
| `initial_sp` [initial_sp_verus.rs](initial_sp_verus.rs) | EXTRA_IN_VERUS |  | 403-413 |
| `num_pages` [num_pages_verus.rs](num_pages_verus.rs) | EXTRA_IN_VERUS |  | 332-342 |
| `page_index` [page_index_verus.rs](page_index_verus.rs) | EXTRA_IN_VERUS |  | 376-388 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | MISMATCH | ❌ |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `size` [size.diff](size.diff) | [size_source.rs](size_source.rs) | [size_verus.rs](size_verus.rs) | MISMATCH | ❌ |
| `top` [top.diff](top.diff) | [top_source.rs](top_source.rs) | [top_verus.rs](top_verus.rs) | MISMATCH | ❌ |
| `contains` [contains_verus.rs](contains_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `has_room` [has_room_verus.rs](has_room_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `initial_sp` [initial_sp_verus.rs](initial_sp_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `num_pages` [num_pages_verus.rs](num_pages_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `page_index` [page_index_verus.rs](page_index_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `KernelStack` [struct_KernelStack.diff](struct_KernelStack.diff) | [struct_KernelStack_source.rs](struct_KernelStack_source.rs) | [struct_KernelStack_verus.rs](struct_KernelStack_verus.rs): MISMATCH
