# Review: kpage (gpt-5.1-codex-max)

## Grade: A-

## Issues Found

### Medium
- **Location:** impl PartialEq for PageAddress (external_body eq)
  **Status:** Unresolved. Equality remains marked `#[verifier::external_body]` even though the implementation is a simple raw-address comparison. This introduces an unnecessary trust hole.
  **Suggested Fix:** Provide a verified body (e.g., `self.raw_addr == other.raw_addr`) or prove a short lemma that justifies the external call, eliminating the need for `external_body`.

## Resolved Items from Previous Review
- **Identity mapping modeling (base/frame_address/inv):** Resolved. The module now includes explicit justification (comments and `proof_identity_mapping_justification`) and enforces identity mapping via the invariant, matching the kernel’s documented identity mapping behavior.

## Summary
All high-severity concerns about identity mapping equivalence have been addressed with explicit justification and invariant enforcement. One medium-severity issue remains: the unnecessary `external_body` on `PageAddress` equality. Once that is removed or proven, the module would be fully verified without trust gaps.
