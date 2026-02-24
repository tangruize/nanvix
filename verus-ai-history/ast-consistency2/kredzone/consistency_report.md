# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/kredzone.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/kredzone.rs`

## Summary

- Functions matched: 0/2
- Functions mismatched: 2
- Missing in Verus: 0
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `load` [load.diff](load.diff) [load_source.rs](load_source.rs) [load_verus.rs](load_verus.rs) | MISMATCH | 81-96 | 354-369 |
| `store` [store.diff](store.diff) [store_source.rs](store_source.rs) [store_verus.rs](store_verus.rs) | MISMATCH | 49-66 | 270-285 |
| `init_kredzone` [init_kredzone_verus.rs](init_kredzone_verus.rs) | EXTRA_IN_VERUS |  | 511-526 |
| `load_with_ghost` [load_with_ghost_verus.rs](load_with_ghost_verus.rs) | EXTRA_IN_VERUS |  | 468-489 |
| `raw_load` [raw_load_verus.rs](raw_load_verus.rs) | EXTRA_IN_VERUS |  | 388-400 |
| `raw_store` [raw_store_verus.rs](raw_store_verus.rs) | EXTRA_IN_VERUS |  | 304-315 |
| `store_with_ghost` [store_with_ghost_verus.rs](store_with_ghost_verus.rs) | EXTRA_IN_VERUS |  | 422-449 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `load` [load.diff](load.diff) [load_source.rs](load_source.rs) [load_verus.rs](load_verus.rs) | MISMATCH | ❌ |
| `store` [store.diff](store.diff) [store_source.rs](store_source.rs) [store_verus.rs](store_verus.rs) | MISMATCH | ❌ |
| `init_kredzone` [init_kredzone_verus.rs](init_kredzone_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `load_with_ghost` [load_with_ghost_verus.rs](load_with_ghost_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `raw_load` [raw_load_verus.rs](raw_load_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `raw_store` [raw_store_verus.rs](raw_store_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `store_with_ghost` [store_with_ghost_verus.rs](store_with_ghost_verus.rs) | EXTRA_IN_VERUS | ❌ |
