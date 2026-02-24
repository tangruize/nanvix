# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/hal/mem/types/address/frame.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/hal/mem/types/address/frame.rs`

## Summary

- Functions matched: 0/9
- Functions mismatched: 5
- Missing in Verus: 4
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `eq` [eq_source.rs](eq_source.rs) | MISSING_IN_VERUS | 79-81 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 73-75 |  |
| `from_frame_number` [from_frame_number.diff](from_frame_number.diff) [from_frame_number_source.rs](from_frame_number_source.rs) [from_frame_number_verus.rs](from_frame_number_verus.rs) | MISMATCH | 46-48 | 89-113 |
| `from_raw_value` [from_raw_value.diff](from_raw_value.diff) [from_raw_value_source.rs](from_raw_value_source.rs) [from_raw_value_verus.rs](from_raw_value_verus.rs) | MISMATCH | 54-56 | 150-164 |
| `into_frame_number` [into_frame_number.diff](into_frame_number.diff) [into_frame_number_source.rs](into_frame_number_source.rs) [into_frame_number_verus.rs](into_frame_number_verus.rs) | MISMATCH | 50-52 | 168-175 |
| `into_page_address` [into_page_address_source.rs](into_page_address_source.rs) | MISSING_IN_VERUS | 42-44 |  |
| `into_physical_address` [into_physical_address.diff](into_physical_address.diff) [into_physical_address_source.rs](into_physical_address_source.rs) [into_physical_address_verus.rs](into_physical_address_verus.rs) | MISSING_IN_VERUS | 38-40 |  |
| `into_raw_value` [into_raw_value.diff](into_raw_value.diff) [into_raw_value_source.rs](into_raw_value_source.rs) [into_raw_value_verus.rs](into_raw_value_verus.rs) | MISMATCH | 67-69 | 128-133 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 34-36 | 212-236 |
| `frame_count` [frame_count_verus.rs](frame_count_verus.rs) | EXTRA_IN_VERUS |  | 265-270 |
| `size` [size_verus.rs](size_verus.rs) | EXTRA_IN_VERUS |  | 256-261 |
| `start` [start_verus.rs](start_verus.rs) | EXTRA_IN_VERUS |  | 240-252 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `eq` [eq_source.rs](eq_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `from_frame_number` [from_frame_number.diff](from_frame_number.diff) [from_frame_number_source.rs](from_frame_number_source.rs) [from_frame_number_verus.rs](from_frame_number_verus.rs) | MISMATCH | ❌ |
| `from_raw_value` [from_raw_value.diff](from_raw_value.diff) [from_raw_value_source.rs](from_raw_value_source.rs) [from_raw_value_verus.rs](from_raw_value_verus.rs) | MISMATCH | ❌ |
| `into_frame_number` [into_frame_number.diff](into_frame_number.diff) [into_frame_number_source.rs](into_frame_number_source.rs) [into_frame_number_verus.rs](into_frame_number_verus.rs) | MISMATCH | ❌ |
| `into_page_address` [into_page_address_source.rs](into_page_address_source.rs) | MISSING_IN_VERUS | ❌ |
| `into_physical_address` [into_physical_address.diff](into_physical_address.diff) [into_physical_address_source.rs](into_physical_address_source.rs) [into_physical_address_verus.rs](into_physical_address_verus.rs) | MISSING_IN_VERUS | ❌ |
| `into_raw_value` [into_raw_value.diff](into_raw_value.diff) [into_raw_value_source.rs](into_raw_value_source.rs) [into_raw_value_verus.rs](into_raw_value_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `frame_count` [frame_count_verus.rs](frame_count_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `size` [size_verus.rs](size_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `start` [start_verus.rs](start_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `FrameAddress` [struct_FrameAddress.diff](struct_FrameAddress.diff) [struct_FrameAddress_source.rs](struct_FrameAddress_source.rs) [struct_FrameAddress_verus.rs](struct_FrameAddress_verus.rs): MISMATCH
- `FrameNumber` [struct_FrameNumber_verus.rs](struct_FrameNumber_verus.rs): EXTRA_IN_VERUS
- `PageAlignedPhysAddr` [struct_PageAlignedPhysAddr_verus.rs](struct_PageAlignedPhysAddr_verus.rs): EXTRA_IN_VERUS
- `TruncatedMemoryRegion` [struct_TruncatedMemoryRegion_verus.rs](struct_TruncatedMemoryRegion_verus.rs): EXTRA_IN_VERUS
