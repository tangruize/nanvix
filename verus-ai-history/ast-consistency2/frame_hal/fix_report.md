# Exec Consistency Fix: frame_hal

## Summary
- Mismatches fixed: 0 (5 documented as semantically equivalent)
- Missing functions added: 3 (`new`, `into_physical_address`, `from_raw_value`, `eq` via derive)
- Documented equivalences: 5
- Documented omissions: 2 (`into_page_address`, `fmt`)
- Documented extras: 3 types + 3 functions (verification helpers)
- Review issues addressed: 6

## Review Issues Addressed

| Issue # | Severity | Description | Resolution |
|---------|----------|-------------|------------|
| 1 | Major | MAX_FRAME_NUMBER off-by-one (missing `- 1`) | Fixed: changed to `0xFFFF_FFFF / FRAME_SIZE - 1` to match original `FrameNumber::MAX = mem::MAX_ADDRESS / mem::FRAME_SIZE - 1`. |
| 2 | Major | `from_raw_value` omits address-range validation | Documented as intentional: `MEMORY_SIZE` is a kernel configuration constant not in Verus scope. The alignment check is the frame-address-specific validation; bounds checking belongs to the `PhysicalAddress` layer. Added `# Deviation` doc section. |
| 3 | Minor | `FrameNumber` invariant trivially `true` | Fixed: strengthened `inv()` to `self.value as int <= MAX_FRAME_NUMBER as int`. Added corresponding preconditions to `into_frame_number` on both `FrameAddress` and `PageAlignedPhysAddr` (mirrors original's implicit bound from `FrameNumber::from_raw_value().unwrap()`). |
| 4 | Minor | `pub` fields weaken encapsulation | Documented: added `# Note on pub fields` sections to `FrameNumber`, `FrameAddress`, and `PageAlignedPhysAddr` invariant docs explaining the Verus limitation and recommending constructor usage. |
| 5 | Minor | `PartialEq` derive vs manual impl | Accepted as equivalent (reviewer concurred). No change needed. |
| 6 | Minor | `from_frame_number` error code differs (`BadAddress` vs `InvalidArgument`) | Documented: added `# Deviation` section explaining the error code difference. With the precondition satisfied, the error path is unreachable in both versions. |

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `MAX_FRAME_NUMBER` | FIXED | Added missing `- 1` to match original `FrameNumber::MAX`. |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | ADDED | Original `FrameAddress::new(PageAligned<PhysicalAddress>)` mapped to `new(PageAlignedPhysAddr)`. Verified with `requires address.inv()`, `ensures result.inv()`. |
| `into_physical_address` [into_physical_address.diff](into_physical_address.diff) [into_physical_address_source.rs](into_physical_address_source.rs) [into_physical_address_verus.rs](into_physical_address_verus.rs) | ADDED | Original returns `PageAligned<PhysicalAddress>`, Verus returns `PageAlignedPhysAddr`. Both unwrap the inner address. Verified. |
| `from_raw_value` [from_raw_value.diff](from_raw_value.diff) [from_raw_value_source.rs](from_raw_value_source.rs) [from_raw_value_verus.rs](from_raw_value_verus.rs) | ADDED to FrameAddress | Alignment check matches original; address-range omission documented as intentional. Verified. |
| `eq` (PartialEq) | ADDED via `#[derive(PartialEq)]` | Original compares `self.0 == other.0`; derive compares `self.raw_addr == other.raw_addr`. Equivalent for single-field struct. |
| `from_frame_number` [from_frame_number.diff](from_frame_number.diff) [from_frame_number_source.rs](from_frame_number_source.rs) [from_frame_number_verus.rs](from_frame_number_verus.rs) | DOCUMENTED | Error code deviation documented. Exec logic equivalent. |
| `into_frame_number` [into_frame_number.diff](into_frame_number.diff) [into_frame_number_source.rs](into_frame_number_source.rs) [into_frame_number_verus.rs](into_frame_number_verus.rs) | STRENGTHENED | Added precondition `self.spec_raw_value() / FRAME_SIZE <= MAX_FRAME_NUMBER` to both `FrameAddress` and `PageAlignedPhysAddr` versions, matching original's implicit bound guarantee. |
| `into_raw_value` [into_raw_value.diff](into_raw_value.diff) [into_raw_value_source.rs](into_raw_value_source.rs) [into_raw_value_verus.rs](into_raw_value_verus.rs) | DOCUMENTED | Semantically equivalent. |
| `into_page_address` [into_page_address_source.rs](into_page_address_source.rs) | OMITTED | Cross-module dependency on `PageAddress`. Documented. |
| `fmt` (Debug) | OMITTED | Cosmetic difference only. Documented. |
| `FrameNumber::inv` | STRENGTHENED | Changed from `true` to `self.value <= MAX_FRAME_NUMBER`. |
| `FrameNumber` struct | DOCUMENTED (extra) | Replicated from `arch::mem::paging`; exec logic identical. |
| `PageAlignedPhysAddr` struct | DOCUMENTED (extra) | Verification helper replacing `PageAligned<PhysicalAddress>`. |
| `TruncatedMemoryRegion` struct | DOCUMENTED (extra) | Verification helper for frame allocator. |
| `start`, `size`, `frame_count` | DOCUMENTED (extra) | Accessor methods for `TruncatedMemoryRegion`. |

## Verification: PASS

- Before: 18 verified, 0 errors
- After: 21 verified, 0 errors (+3 from `new`, `into_physical_address`, `from_raw_value`)
- No `assume`, `admit`, or unjustified `external_body` used
