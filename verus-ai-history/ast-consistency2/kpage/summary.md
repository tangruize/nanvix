# Exec Diff: kpage

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/virt/kpage.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/virt/kpage.rs`

| Function | Status | Files |
|----------|--------|-------|
| `base` | MISMATCH | base_source.rs, base_verus.rs, base.diff |
| `frame_address` | MISMATCH | frame_address_source.rs, frame_address_verus.rs, frame_address.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `eq` | EXTRA_IN_VERUS | eq_verus.rs (EXTRA) |
| `get_pte_index` | EXTRA_IN_VERUS | get_pte_index_verus.rs (EXTRA) |
| `into_raw_value` | EXTRA_IN_VERUS | into_raw_value_verus.rs (EXTRA) |
| `page_address_eq` | EXTRA_IN_VERUS | page_address_eq_verus.rs (EXTRA) |
| `pool_id` | EXTRA_IN_VERUS | pool_id_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `PageAddress` | EXTRA_IN_VERUS | struct_PageAddress_verus.rs (EXTRA) |
