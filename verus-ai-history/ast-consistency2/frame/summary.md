# Exec Diff: frame

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/frame.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/frame.rs`

| Function | Status | Files |
|----------|--------|-------|
| `alloc` | MISMATCH | alloc_source.rs, alloc_verus.rs, alloc.diff |
| `alloc_range` | MISMATCH | alloc_range_source.rs, alloc_range_verus.rs, alloc_range.diff |
| `book` | MISMATCH | book_source.rs, book_verus.rs, book.diff |
| `free` | MISMATCH | free_source.rs, free_verus.rs, free.diff |
| `from_raw_storage` | MISMATCH | from_raw_storage_source.rs, from_raw_storage_verus.rs, from_raw_storage.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `alloc_contiguous_range` | EXTRA_IN_VERUS | alloc_contiguous_range_verus.rs (EXTRA) |
| `alloc_index` | EXTRA_IN_VERUS | alloc_index_verus.rs (EXTRA) |
| `alloc_range_checked` | EXTRA_IN_VERUS | alloc_range_checked_verus.rs (EXTRA) |
| `alloc_range_inner` | EXTRA_IN_VERUS | alloc_range_inner_verus.rs (EXTRA) |
| `alloc_range_unchecked` | EXTRA_IN_VERUS | alloc_range_unchecked_verus.rs (EXTRA) |
| `capacity` | EXTRA_IN_VERUS | capacity_verus.rs (EXTRA) |
| `free_range` | EXTRA_IN_VERUS | free_range_verus.rs (EXTRA) |
| `free_range_inner` | EXTRA_IN_VERUS | free_range_inner_verus.rs (EXTRA) |
