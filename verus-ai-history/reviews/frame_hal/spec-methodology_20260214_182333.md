# Review: frame_hal Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **No View types defined (Criterion 1, 4):** The methodology (Step 1) requires defining a `MyTypeView` abstraction type using abstract types (`int`, `Seq`, `Set`, `Map`) and implementing `pub closed spec fn view(&self) -> MyTypeView`. None of the four types (`FrameNumber`, `FrameAddress`, `PageAlignedPhysAddr`, `TruncatedMemoryRegion`) define a View type or implement `view()`. Instead, the code uses multiple `pub open spec fn` helpers (e.g., `spec_raw_value()`, `spec_frame_number()`) to expose individual abstract fields. While this achieves a similar effect, it deviates from the prescribed methodology pattern and prevents use of the `self@.field` shorthand in public method specifications.
- **Public struct fields expose implementation details (Criterion 4):** `FrameNumber.value`, `FrameAddress.raw_addr`, and `PageAlignedPhysAddr.raw_addr` are all `pub` fields (frame.rs lines 39, 77, 134). This violates the methodology principle that public method specs should not reference `self.field` directly. While the spec functions use `self.value` and `self.raw_addr` internally (which is acceptable within `closed spec fn` definitions), the public visibility of the fields means external code can bypass the abstraction layer entirely.

### Medium
- **Several spec functions are `pub open` instead of `pub closed` (Criterion 2):** `FrameNumber::spec_raw_value()`, `FrameAddress::spec_raw_value()`, `FrameAddress::spec_frame_number()`, `FrameAddress::spec_is_aligned()`, `PageAlignedPhysAddr::spec_raw_value()`, and `PageAlignedPhysAddr::spec_frame_number()` are all `pub open spec fn`. The methodology states that besides `inv()` and `view()`, no further `pub` spec functions should be defined on the impl. If these are needed by external callers, they should be defined on the View type (which doesn't exist yet). If they must remain, they should be `pub closed` to hide implementation details, unless there is a justified reason for openness (e.g., enabling external proofs to unfold definitions).
- **Missing `inv()` in some public method requires/ensures (Criterion 5):**
  - `FrameNumber::from_raw_value()` (line 46): Does not require `inv()` on inputs (N/A for constructor), but the ensures clause correctly guarantees `frame.inv()` on the output. ✓
  - `FrameNumber::into_raw_value()` (line 64): Does not require `self.inv()`. Since `inv()` is trivially `true`, this is harmless but inconsistent with the methodology pattern.
  - `FrameAddress::from_frame_number()` (line 84): Does not require `frame_number.inv()` on the input `FrameNumber` parameter. Ensures `addr.inv()` on output. ✓ (partial)
  - `FrameAddress::into_frame_number()` (line 111): Requires `self.spec_is_aligned()` instead of `self.inv()`. Since `inv()` is equivalent to alignment, this works but is inconsistent with the methodology pattern of using `inv()`.
  - `FrameAddress::into_raw_value()` (line 122): Does not require `self.inv()`.
  - `PageAlignedPhysAddr::from_raw_value()` (line 141): Ensures `pa.inv()` on output. ✓
  - `PageAlignedPhysAddr::into_frame_number()` (line 159): Requires `self.inv()`. ✓
  - `TruncatedMemoryRegion::start()` (line 229): Does not require `self.inv()`.
  - `TruncatedMemoryRegion::size()` (line 243): Does not require `self.inv()`.

### Low
- **Comment references field name directly in doc comment (line 83):** `frame_number.value <= MAX_FRAME_NUMBER` in the doc comment exposes the internal field name. Should reference the spec abstraction instead: `frame_number.spec_raw_value() <= MAX_FRAME_NUMBER`.
- **`TruncatedMemoryRegion` fields are private but `start` and `size` fields of other structs are public:** Inconsistent visibility across the types. `TruncatedMemoryRegion` correctly has private fields (lines 186-188), which is the right pattern per the methodology. The other structs should follow suit.

## Positive Observations
- **`inv()` is correctly `pub closed spec fn` for all four types (Criterion 3).** ✓
- **No `assume`, `admit`, or `external_body` found anywhere (Criterion 6).** ✓
- **Verification passes: 18 verified, 0 errors (Criterion 7).** ✓
- **Abstract types used correctly in spec functions:** All spec functions return `int` or `bool`, not concrete types like `usize`. ✓
- **Proof file is clean and minimal:** The single lemma `lemma_inv_implies_frame_count_positive` is well-justified and bridges the gap between the closed `inv()` and the property callers need. ✓
- **Error handling follows methodology:** Constructors return `Result` or `Option` with clear ensures clauses covering both success and failure cases. ✓

## Summary

The frame_hal verification is solid with clean verification (18/0), no assumptions or admits, and correct use of `pub closed spec fn` for all `inv()` functions. The main methodology gap is the absence of View types — the code uses individual `pub open spec fn` helpers instead of the prescribed `MyTypeView` + `view()` pattern from the guidelines. This means `self@.field` notation is never used in public method specs, and several spec functions are unnecessarily `pub open`, potentially leaking implementation details. Additionally, some public struct fields are exposed (FrameNumber, FrameAddress, PageAlignedPhysAddr), and `inv()` is not consistently required/ensured across all public methods. These are methodology conformance issues rather than correctness bugs — the verification itself is sound. Grade B+ reflects strong verification results with notable methodology deviations.
