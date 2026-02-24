# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/phys/frame.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/phys/frame.rs`

## Summary

- Functions matched: 0/6
- Functions mismatched: 5
- Missing in Verus: 1
- Extra in Verus: 8
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc` [alloc.diff](alloc.diff) [alloc_source.rs](alloc_source.rs) [alloc_verus.rs](alloc_verus.rs) | MISMATCH | 85-110 | 228-293 |
| `alloc_range` [alloc_range.diff](alloc_range.diff) [alloc_range_source.rs](alloc_range_source.rs) [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | 174-201 | 618-748 |
| `book` [book.diff](book.diff) [book_source.rs](book_source.rs) [book_verus.rs](book_verus.rs) | MISMATCH | 149-158 | 368-416 |
| `free` [free.diff](free.diff) [free_source.rs](free_source.rs) [free_verus.rs](free_verus.rs) | MISMATCH | 125-134 | 307-349 |
| `from_raw_storage` [from_raw_storage.diff](from_raw_storage.diff) [from_raw_storage_source.rs](from_raw_storage_source.rs) [from_raw_storage_verus.rs](from_raw_storage_verus.rs) | MISSING_IN_VERUS | 71-73 |  |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 59-69 | 71-97 |
| `alloc_contiguous_range` [alloc_contiguous_range_verus.rs](alloc_contiguous_range_verus.rs) | EXTRA_IN_VERUS |  | 842-918 |
| `alloc_index` [alloc_index_verus.rs](alloc_index_verus.rs) | EXTRA_IN_VERUS |  | 170-215 |
| `alloc_range_from_region` | EXTRA_IN_VERUS |  | 767-812 |
| `alloc_range_inner` [alloc_range_inner_verus.rs](alloc_range_inner_verus.rs) | EXTRA_IN_VERUS |  | 523-595 |
| `alloc_range_unchecked` [alloc_range_unchecked_verus.rs](alloc_range_unchecked_verus.rs) | EXTRA_IN_VERUS |  | 444-513 |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | EXTRA_IN_VERUS |  | 147-155 |
| `free_range` [free_range_verus.rs](free_range_verus.rs) | EXTRA_IN_VERUS |  | 940-1006 |
| `free_range_inner` [free_range_inner_verus.rs](free_range_inner_verus.rs) | EXTRA_IN_VERUS |  | 1010-1078 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc` [alloc.diff](alloc.diff) [alloc_source.rs](alloc_source.rs) [alloc_verus.rs](alloc_verus.rs) | MISMATCH | ❌ |
| `alloc_range` [alloc_range.diff](alloc_range.diff) [alloc_range_source.rs](alloc_range_source.rs) [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | ❌ |
| `book` [book.diff](book.diff) [book_source.rs](book_source.rs) [book_verus.rs](book_verus.rs) | MISMATCH | ❌ |
| `free` [free.diff](free.diff) [free_source.rs](free_source.rs) [free_verus.rs](free_verus.rs) | MISMATCH | ❌ |
| `from_raw_storage` [from_raw_storage.diff](from_raw_storage.diff) [from_raw_storage_source.rs](from_raw_storage_source.rs) [from_raw_storage_verus.rs](from_raw_storage_verus.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `alloc_contiguous_range` [alloc_contiguous_range_verus.rs](alloc_contiguous_range_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_index` [alloc_index_verus.rs](alloc_index_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_range_from_region` | EXTRA_IN_VERUS | ❌ |
| `alloc_range_inner` [alloc_range_inner_verus.rs](alloc_range_inner_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `alloc_range_unchecked` [alloc_range_unchecked_verus.rs](alloc_range_unchecked_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `free_range` [free_range_verus.rs](free_range_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `free_range_inner` [free_range_inner_verus.rs](free_range_inner_verus.rs) | EXTRA_IN_VERUS | ❌ |
