# Review: kpage (gpt-5.1-codex-max)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** KernelPage::base, KernelPage::frame_address, KernelPage::inv
  **Description:** Verification hardcodes identity mapping (page_addr == frame_addr) and derives the page address directly from the frame raw address. The original code composes `frame.base().into_page_address().into_virtual_address()`, which may apply a physical→virtual translation (e.g., higher-half offset) rather than identity. If the real kernel uses an offset mapping, the verified model proves properties for a stronger/incorrect abstraction and is not semantically equivalent.
  **Suggested Fix:** Model the actual physical-to-virtual translation used by the kernel (e.g., add a spec/constant for the kernel offset and construct PageAddress via that mapping), and prove base/frame_address consistency using that mapping. Alternatively, add a requires/assumes clause that explicitly states kernel memory is identity-mapped and ensure this matches the real kernel configuration.

### Medium
- **Location:** impl PartialEq for PageAddress (external_body eq)
  **Description:** Equality is marked `external_body`, leaving it unproved and trusted. Since it is a simple raw address comparison, this trust hole is unnecessary.
  **Suggested Fix:** Provide a verified body (e.g., `self.raw_addr == other.raw_addr`) or a small proof lemma to justify equality without `external_body`.

### Low
- None.

## Positive Observations
- Coverage matches the original API: `new`, `base`, and `frame_address` are all present and verified with alignment/provenance invariants.
- Invariants capture alignment, address consistency, and pool provenance, and the verifier fully discharges the module (13 goals, 0 errors).

## Summary
The verification is close but assumes identity mapping, which may diverge from the original physical-to-virtual translation path, making the proof potentially non-equivalent. Align the verified model with the actual kernel mapping (or make the assumption explicit and true) and remove the unnecessary `external_body` on equality to eliminate the remaining soundness gap.
