# Exec Diff: tid

**Source:** `/home/ubuntu/nanvix/src/libs/sys/src/sys/pm/tid.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/sys/sys/pm/tid.rs`

| Function | Status | Files |
|----------|--------|-------|
| `fmt` | MISMATCH | fmt_source.rs, fmt_verus.rs, fmt.diff |
| `from` | MISMATCH | from_source.rs, from_verus.rs, from.diff |
| `from_ne_bytes` | MISMATCH | from_ne_bytes_source.rs, from_ne_bytes_verus.rs, from_ne_bytes.diff |
| `to_ne_bytes` | MISMATCH | to_ne_bytes_source.rs, to_ne_bytes_verus.rs, to_ne_bytes.diff |
| `try_from` | MISMATCH | try_from_source.rs, try_from_verus.rs, try_from.diff |
| `cmp` | EXTRA_IN_VERUS | cmp_verus.rs (EXTRA) |
| `cmp_ord` | EXTRA_IN_VERUS | cmp_ord_verus.rs (EXTRA) |
| `default` | EXTRA_IN_VERUS | default_verus.rs (EXTRA) |
| `default_value` | EXTRA_IN_VERUS | default_value_verus.rs (EXTRA) |
| `eq` | EXTRA_IN_VERUS | eq_verus.rs (EXTRA) |
| `from_i32` | EXTRA_IN_VERUS | from_i32_verus.rs (EXTRA) |
| `ge` | EXTRA_IN_VERUS | ge_verus.rs (EXTRA) |
| `gt` | EXTRA_IN_VERUS | gt_verus.rs (EXTRA) |
| `into_i32` | EXTRA_IN_VERUS | into_i32_verus.rs (EXTRA) |
| `into_i64` | EXTRA_IN_VERUS | into_i64_verus.rs (EXTRA) |
| `into_isize` | EXTRA_IN_VERUS | into_isize_verus.rs (EXTRA) |
| `le` | EXTRA_IN_VERUS | le_verus.rs (EXTRA) |
| `lt` | EXTRA_IN_VERUS | lt_verus.rs (EXTRA) |
| `ne` | EXTRA_IN_VERUS | ne_verus.rs (EXTRA) |
| `partial_cmp` | EXTRA_IN_VERUS | partial_cmp_verus.rs (EXTRA) |
| `try_from_i64` | EXTRA_IN_VERUS | try_from_i64_verus.rs (EXTRA) |
| `try_from_isize` | EXTRA_IN_VERUS | try_from_isize_verus.rs (EXTRA) |
| `try_from_u32` | EXTRA_IN_VERUS | try_from_u32_verus.rs (EXTRA) |
| `try_from_u64` | EXTRA_IN_VERUS | try_from_u64_verus.rs (EXTRA) |
| `try_from_usize` | EXTRA_IN_VERUS | try_from_usize_verus.rs (EXTRA) |
| `try_into_u32` | EXTRA_IN_VERUS | try_into_u32_verus.rs (EXTRA) |
| `try_into_u64` | EXTRA_IN_VERUS | try_into_u64_verus.rs (EXTRA) |
| `try_into_usize` | EXTRA_IN_VERUS | try_into_usize_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ThreadIdentifier` | MISMATCH | struct_ThreadIdentifier_source.rs, struct_ThreadIdentifier_verus.rs, struct_ThreadIdentifier.diff |
