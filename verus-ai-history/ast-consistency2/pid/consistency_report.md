# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/sys/src/sys/pm/pid.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/sys/sys/pm/pid.rs`

## Summary

- Functions matched: 0/5
- Functions mismatched: 5
- Missing in Verus: 0
- Extra in Verus: 23
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | MISMATCH | 191-193 | 646-648 |
| `from` [from.diff](from.diff) | [from_source.rs](from_source.rs) | [from_verus.rs](from_verus.rs) | MISMATCH | 133-135 | 674-676 |
| `from_ne_bytes` [from_ne_bytes.diff](from_ne_bytes.diff) | [from_ne_bytes_source.rs](from_ne_bytes_source.rs) | [from_ne_bytes_verus.rs](from_ne_bytes_verus.rs) | MISMATCH | 66-68 | 434-440 |
| `to_ne_bytes` [to_ne_bytes.diff](to_ne_bytes.diff) | [to_ne_bytes_source.rs](to_ne_bytes_source.rs) | [to_ne_bytes_verus.rs](to_ne_bytes_verus.rs) | MISMATCH | 62-64 | 410-417 |
| `try_from` [try_from.diff](try_from.diff) | [try_from_source.rs](try_from_source.rs) | [try_from_verus.rs](try_from_verus.rs) | MISMATCH | 180-187 | 746-748 |
| `cmp` [cmp_verus.rs](cmp_verus.rs) | EXTRA_IN_VERUS |  | 639-641 |
| `cmp_ord` [cmp_ord_verus.rs](cmp_ord_verus.rs) | EXTRA_IN_VERUS |  | 571-587 |
| `default` [default_verus.rs](default_verus.rs) | EXTRA_IN_VERUS |  | 616-618 |
| `default_value` [default_value_verus.rs](default_value_verus.rs) | EXTRA_IN_VERUS |  | 595-602 |
| `eq` [eq_verus.rs](eq_verus.rs) | EXTRA_IN_VERUS |  | 623-625 |
| `from_i32` [from_i32_verus.rs](from_i32_verus.rs) | EXTRA_IN_VERUS |  | 95-102 |
| `ge` [ge_verus.rs](ge_verus.rs) | EXTRA_IN_VERUS |  | 552-560 |
| `gt` [gt_verus.rs](gt_verus.rs) | EXTRA_IN_VERUS |  | 532-540 |
| `into_i32` [into_i32_verus.rs](into_i32_verus.rs) | EXTRA_IN_VERUS |  | 110-117 |
| `into_i64` [into_i64_verus.rs](into_i64_verus.rs) | EXTRA_IN_VERUS |  | 140-147 |
| `into_isize` [into_isize_verus.rs](into_isize_verus.rs) | EXTRA_IN_VERUS |  | 125-132 |
| `le` [le_verus.rs](le_verus.rs) | EXTRA_IN_VERUS |  | 512-520 |
| `lt` [lt_verus.rs](lt_verus.rs) | EXTRA_IN_VERUS |  | 492-500 |
| `ne` [ne_verus.rs](ne_verus.rs) | EXTRA_IN_VERUS |  | 472-480 |
| `partial_cmp` [partial_cmp_verus.rs](partial_cmp_verus.rs) | EXTRA_IN_VERUS |  | 632-634 |
| `try_from_i64` [try_from_i64_verus.rs](try_from_i64_verus.rs) | EXTRA_IN_VERUS |  | 281-298 |
| `try_from_isize` [try_from_isize_verus.rs](try_from_isize_verus.rs) | EXTRA_IN_VERUS |  | 249-266 |
| `try_from_u32` [try_from_u32_verus.rs](try_from_u32_verus.rs) | EXTRA_IN_VERUS |  | 346-364 |
| `try_from_u64` [try_from_u64_verus.rs](try_from_u64_verus.rs) | EXTRA_IN_VERUS |  | 379-397 |
| `try_from_usize` [try_from_usize_verus.rs](try_from_usize_verus.rs) | EXTRA_IN_VERUS |  | 313-331 |
| `try_into_u32` [try_into_u32_verus.rs](try_into_u32_verus.rs) | EXTRA_IN_VERUS |  | 187-205 |
| `try_into_u64` [try_into_u64_verus.rs](try_into_u64_verus.rs) | EXTRA_IN_VERUS |  | 216-234 |
| `try_into_usize` [try_into_usize_verus.rs](try_into_usize_verus.rs) | EXTRA_IN_VERUS |  | 158-176 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | MISMATCH | ❌ |
| `from` [from.diff](from.diff) | [from_source.rs](from_source.rs) | [from_verus.rs](from_verus.rs) | MISMATCH | ❌ |
| `from_ne_bytes` [from_ne_bytes.diff](from_ne_bytes.diff) | [from_ne_bytes_source.rs](from_ne_bytes_source.rs) | [from_ne_bytes_verus.rs](from_ne_bytes_verus.rs) | MISMATCH | ❌ |
| `to_ne_bytes` [to_ne_bytes.diff](to_ne_bytes.diff) | [to_ne_bytes_source.rs](to_ne_bytes_source.rs) | [to_ne_bytes_verus.rs](to_ne_bytes_verus.rs) | MISMATCH | ❌ |
| `try_from` [try_from.diff](try_from.diff) | [try_from_source.rs](try_from_source.rs) | [try_from_verus.rs](try_from_verus.rs) | MISMATCH | ❌ |
| `cmp` [cmp_verus.rs](cmp_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `cmp_ord` [cmp_ord_verus.rs](cmp_ord_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `default` [default_verus.rs](default_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `default_value` [default_value_verus.rs](default_value_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `eq` [eq_verus.rs](eq_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `from_i32` [from_i32_verus.rs](from_i32_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `ge` [ge_verus.rs](ge_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `gt` [gt_verus.rs](gt_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `into_i32` [into_i32_verus.rs](into_i32_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `into_i64` [into_i64_verus.rs](into_i64_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `into_isize` [into_isize_verus.rs](into_isize_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `le` [le_verus.rs](le_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `lt` [lt_verus.rs](lt_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `ne` [ne_verus.rs](ne_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `partial_cmp` [partial_cmp_verus.rs](partial_cmp_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_from_i64` [try_from_i64_verus.rs](try_from_i64_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_from_isize` [try_from_isize_verus.rs](try_from_isize_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_from_u32` [try_from_u32_verus.rs](try_from_u32_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_from_u64` [try_from_u64_verus.rs](try_from_u64_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_from_usize` [try_from_usize_verus.rs](try_from_usize_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_into_u32` [try_into_u32_verus.rs](try_into_u32_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_into_u64` [try_into_u64_verus.rs](try_into_u64_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `try_into_usize` [try_into_usize_verus.rs](try_into_usize_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ProcessIdentifier` [struct_ProcessIdentifier.diff](struct_ProcessIdentifier.diff) | [struct_ProcessIdentifier_source.rs](struct_ProcessIdentifier_source.rs) | [struct_ProcessIdentifier_verus.rs](struct_ProcessIdentifier_verus.rs): MISMATCH
