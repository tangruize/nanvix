# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/virt/kpage.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/virt/kpage.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 3
- Missing in Verus: 0
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `base` [base.diff](base.diff) [base_source.rs](base_source.rs) [base_verus.rs](base_verus.rs) | MISMATCH | 57-65 | 350-365 |
| `frame_address` [frame_address.diff](frame_address.diff) [frame_address_source.rs](frame_address_source.rs) [frame_address_verus.rs](frame_address_verus.rs) | MISMATCH | 76-78 | 377-389 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 44-46 | 316-331 |
| `eq` [eq_verus.rs](eq_verus.rs) | EXTRA_IN_VERUS |  | 241-243 |
| `get_pte_index` [get_pte_index_verus.rs](get_pte_index_verus.rs) | EXTRA_IN_VERUS |  | 171-181 |
| `into_raw_value` [into_raw_value_verus.rs](into_raw_value_verus.rs) | EXTRA_IN_VERUS |  | 151-157 |
| `page_address_eq` [page_address_eq_verus.rs](page_address_eq_verus.rs) | EXTRA_IN_VERUS |  | 212-221 |
| `pool_id` [pool_id_verus.rs](pool_id_verus.rs) | EXTRA_IN_VERUS |  | 403-410 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `base` [base.diff](base.diff) [base_source.rs](base_source.rs) [base_verus.rs](base_verus.rs) | MISMATCH | ❌ |
| `frame_address` [frame_address.diff](frame_address.diff) [frame_address_source.rs](frame_address_source.rs) [frame_address_verus.rs](frame_address_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `eq` [eq_verus.rs](eq_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `get_pte_index` [get_pte_index_verus.rs](get_pte_index_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `into_raw_value` [into_raw_value_verus.rs](into_raw_value_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `page_address_eq` [page_address_eq_verus.rs](page_address_eq_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pool_id` [pool_id_verus.rs](pool_id_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `PageAddress` [struct_PageAddress_verus.rs](struct_PageAddress_verus.rs): EXTRA_IN_VERUS
