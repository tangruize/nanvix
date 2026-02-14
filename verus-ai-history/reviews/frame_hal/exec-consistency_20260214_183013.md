# Review: frame_hal Exec Consistency (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- None.

### Major

1. **MAX_FRAME_NUMBER diverges from original FrameNumber::MAX.**
   - Original: `FrameNumber::MAX = mem::MAX_ADDRESS / mem::FRAME_SIZE - 1` where `MAX_ADDRESS = usize::MAX`. On the target 32-bit x86 platform this yields `0xFFFF_FFFF / 4096 - 1 = 0x000F_FFFF - 1 = 0x000F_FFFE` (1,048,574).
   - Verus: `MAX_FRAME_NUMBER = 0xFFFF_FFFF / FRAME_SIZE` = `0x000F_FFFF` (1,048,575).
   - The off-by-one (`- 1`) is missing in the Verus version. This means the Verus code accepts one additional frame number value that the original rejects. While unlikely to cause a verification unsoundness, it is a semantic divergence from the original that should be documented or corrected.

2. **`from_raw_value` omits address-range validation present in original.**
   - Original chain: `PhysicalAddress::from_raw_value(raw_addr)?` delegates to `PhysicalAddress::from_virtual_address()`, which checks `addr >= config::kernel::MEMORY_SIZE` and returns `BadAddress` if out of bounds. The Verus version only checks page-alignment, not address-range bounds.
   - This means the Verus `from_raw_value` succeeds for addresses the original would reject (e.g., addresses beyond physical memory size). This is a meaningful behavioral divergence that should be documented as an intentional simplification or addressed.

### Minor

3. **`FrameNumber` invariant is trivially `true`, unlike original.**
   - Original `FrameNumber::from_raw_value` returns `None` if `value > Self::MAX`. The Verus `inv()` spec is `true` (any usize is valid), meaning the invariant does not capture the domain constraint. The construction-time check exists but the invariant does not enforce the bound post-construction, so direct struct literal construction (enabled by `pub value`) can create invalid instances. This weakens verification guarantees.

4. **`pub` fields weaken encapsulation.**
   - `FrameNumber.value`, `FrameAddress.raw_addr`, and `PageAlignedPhysAddr.raw_addr` are all `pub`, with a comment saying "Per methodology Step 1, new callers should use from_raw_value() instead." However, nothing prevents bypassing constructors and creating instances that violate invariants. This is a pragmatic Verus trade-off but should be noted.

5. **`PartialEq` derive vs manual impl — semantically equivalent but not structurally identical.**
   - Original has a manual `impl PartialEq` comparing `self.0 == other.0`. The Verus code uses `#[derive(PartialEq)]` which compares `self.raw_addr == other.raw_addr`. Since the flattened struct has a single field, these are semantically equivalent. Acceptable.

6. **`from_frame_number` error type differs.**
   - Original returns `Result<Self, Error>` and the `?` operator propagates errors from `PageAligned::from_address` (which returns `ErrorCode::BadAddress`). The Verus version returns `ErrorCode::InvalidArgument` with message "frame number overflow". The error code diverges (`BadAddress` vs `InvalidArgument`), though this is unlikely to matter for verification purposes.

### Informational

7. **`into_page_address` omission is well-justified.** It depends on cross-module type `PageAddress` from `kernel::mm::virt::kpage`, which is out of scope. Acceptable.

8. **`Debug` format difference is cosmetic.** Original uses `FrameAddress({:#010x})`, Verus derives `Debug`. No behavioral impact. Acceptable.

9. **Extra types (`PageAlignedPhysAddr`, `TruncatedMemoryRegion`, `FrameNumber`) are well-documented** as verification helpers with clear justifications for their existence.

10. **Proof file is minimal but sound.** The single lemma `lemma_inv_implies_frame_count_positive` correctly derives `frame_count > 0` from the `TruncatedMemoryRegion` invariant. No `assume` or `admit` used.

## Verification Status

- **21 verified, 0 errors** — PASSED.
- No `assume`, `admit`, or unjustified `external_body` used.
- Verification adds 3 new verified items (`new`, `into_physical_address`, `from_raw_value`) over the previous baseline of 18.

## Consistency Assessment

| Original Function | Verus Status | Faithful? |
|---|---|---|
| `new(PageAligned<PhysicalAddress>)` | Added as `new(PageAlignedPhysAddr)` | ✅ Equivalent |
| `into_physical_address` | Added | ✅ Equivalent |
| `from_frame_number` | Present, documented | ⚠️ Error code differs; off-by-one in MAX |
| `into_frame_number` | Present, documented | ✅ Equivalent |
| `from_raw_value` | Added | ⚠️ Missing address-range check |
| `into_raw_value` | Present, documented | ✅ Equivalent |
| `eq` (PartialEq) | Derived | ✅ Equivalent |
| `into_page_address` | Omitted | ✅ Justified (cross-module dep) |
| `fmt` (Debug) | Derived differently | ✅ Cosmetic only |

## Summary

The consistency fixes are generally well-executed. The three added functions (`new`, `into_physical_address`, `from_raw_value`) correctly mirror the original behavior with appropriate Verus specifications. Documentation of equivalences and omissions is thorough. Verification passes cleanly with no admitted proofs.

Two substantive issues prevent a higher grade: (1) the `MAX_FRAME_NUMBER` off-by-one divergence from the original `FrameNumber::MAX`, and (2) the omission of address-range validation in `from_raw_value`. Both are understandable simplifications for verification scope but should be explicitly documented as intentional deviations, especially since the consistency report claims semantic equivalence without noting these differences.
