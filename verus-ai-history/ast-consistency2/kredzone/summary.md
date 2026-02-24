# Exec Diff: kredzone

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/kredzone.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/kredzone.rs`

| Function | Status | Files |
|----------|--------|-------|
| `load` | MISMATCH | load_source.rs, load_verus.rs, load.diff |
| `store` | MISMATCH | store_source.rs, store_verus.rs, store.diff |
| `init_kredzone` | EXTRA_IN_VERUS | init_kredzone_verus.rs (EXTRA) |
| `load_with_ghost` | EXTRA_IN_VERUS | load_with_ghost_verus.rs (EXTRA) |
| `raw_load` | EXTRA_IN_VERUS | raw_load_verus.rs (EXTRA) |
| `raw_store` | EXTRA_IN_VERUS | raw_store_verus.rs (EXTRA) |
| `store_with_ghost` | EXTRA_IN_VERUS | store_with_ghost_verus.rs (EXTRA) |
