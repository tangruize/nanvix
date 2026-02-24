# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/ustack.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/ustack.rs`

## Summary

- Functions matched: 0/5
- Functions mismatched: 4
- Missing in Verus: 1
- Extra in Verus: 10
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | MISMATCH | 61-63 | 498-506 |
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | MISSING_IN_VERUS | 84-92 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 31-33 | 281-351 |
| `size` [size.diff](size.diff) | [size_source.rs](size_source.rs) | [size_verus.rs](size_verus.rs) | MISMATCH | 44-46 | 425-434 |
| `top` [top.diff](top.diff) | [top_source.rs](top_source.rs) | [top_verus.rs](top_verus.rs) | MISMATCH | 78-80 | 513-521 |
| `base_raw` [base_raw_verus.rs](base_raw_verus.rs) | EXTRA_IN_VERUS |  | 453-461 |
| `contains` [contains_verus.rs](contains_verus.rs) | EXTRA_IN_VERUS |  | 535-542 |
| `from_aligned` | EXTRA_IN_VERUS |  | 369-411 |
| `from_raw` [from_raw_verus.rs](from_raw_verus.rs) | EXTRA_IN_VERUS |  | 177-195 |
| `from_raw_unchecked` [from_raw_unchecked_verus.rs](from_raw_unchecked_verus.rs) | EXTRA_IN_VERUS |  | 203-211 |
| `has_room` [has_room_verus.rs](has_room_verus.rs) | EXTRA_IN_VERUS |  | 605-614 |
| `initial_sp` [initial_sp_verus.rs](initial_sp_verus.rs) | EXTRA_IN_VERUS |  | 583-591 |
| `into_raw` [into_raw_verus.rs](into_raw_verus.rs) | EXTRA_IN_VERUS |  | 215-223 |
| `page_index` [page_index_verus.rs](page_index_verus.rs) | EXTRA_IN_VERUS |  | 555-568 |
| `top_raw` [top_raw_verus.rs](top_raw_verus.rs) | EXTRA_IN_VERUS |  | 481-490 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | MISMATCH | ❌ |
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `size` [size.diff](size.diff) | [size_source.rs](size_source.rs) | [size_verus.rs](size_verus.rs) | MISMATCH | ❌ |
| `top` [top.diff](top.diff) | [top_source.rs](top_source.rs) | [top_verus.rs](top_verus.rs) | MISMATCH | ❌ |
| `base_raw` [base_raw_verus.rs](base_raw_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `contains` [contains_verus.rs](contains_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `from_aligned` | EXTRA_IN_VERUS | ❌ |
| `from_raw` [from_raw_verus.rs](from_raw_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `from_raw_unchecked` [from_raw_unchecked_verus.rs](from_raw_unchecked_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `has_room` [has_room_verus.rs](has_room_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `initial_sp` [initial_sp_verus.rs](initial_sp_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `into_raw` [into_raw_verus.rs](into_raw_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `page_index` [page_index_verus.rs](page_index_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `top_raw` [top_raw_verus.rs](top_raw_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `PageAlignedAddr` [struct_PageAlignedAddr_verus.rs](struct_PageAlignedAddr_verus.rs): EXTRA_IN_VERUS
- `UserStack` [struct_UserStack.diff](struct_UserStack.diff) | [struct_UserStack_source.rs](struct_UserStack_source.rs) | [struct_UserStack_verus.rs](struct_UserStack_verus.rs): MISMATCH
