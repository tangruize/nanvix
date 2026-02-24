# Exec Diff: frame

**Source:** `/home/ubuntu/nanvix/src/kernel/src/hal/mem/types/address/frame.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/hal/mem/types/address/frame.rs`

| Function | Status | Files |
|----------|--------|-------|
| `eq` | MISSING_IN_VERUS | eq_source.rs (MISSING in verus) |
| `fmt` | MISSING_IN_VERUS | fmt_source.rs (MISSING in verus) |
| `from_frame_number` | MISMATCH | from_frame_number_source.rs, from_frame_number_verus.rs, from_frame_number.diff |
| `from_raw_value` | MISMATCH | from_raw_value_source.rs, from_raw_value_verus.rs, from_raw_value.diff |
| `into_frame_number` | MISMATCH | into_frame_number_source.rs, into_frame_number_verus.rs, into_frame_number.diff |
| `into_page_address` | MISSING_IN_VERUS | into_page_address_source.rs (MISSING in verus) |
| `into_physical_address` | MISMATCH | into_physical_address_source.rs, into_physical_address_verus.rs, into_physical_address.diff |
| `into_raw_value` | MISMATCH | into_raw_value_source.rs, into_raw_value_verus.rs, into_raw_value.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `frame_count` | EXTRA_IN_VERUS | frame_count_verus.rs (EXTRA) |
| `size` | EXTRA_IN_VERUS | size_verus.rs (EXTRA) |
| `start` | EXTRA_IN_VERUS | start_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `FrameAddress` | MISMATCH | struct_FrameAddress_source.rs, struct_FrameAddress_verus.rs, struct_FrameAddress.diff |
| `FrameNumber` | EXTRA_IN_VERUS | struct_FrameNumber_verus.rs (EXTRA) |
| `PageAlignedPhysAddr` | EXTRA_IN_VERUS | struct_PageAlignedPhysAddr_verus.rs (EXTRA) |
| `TruncatedMemoryRegion` | EXTRA_IN_VERUS | struct_TruncatedMemoryRegion_verus.rs (EXTRA) |
