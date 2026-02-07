# Review: pid (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessIdentifier::assert_layout` (proof, `pid.proof.rs` lines 155-171)  
  **Description:** The layout proof is still never invoked. A search shows no call site in the module or elsewhere, so the size/alignment assertions remain unexercised. This means the ABI/layout requirement is still unenforced in the verified harness.  
  **Status vs previous review:** **Not fixed.** The lemma exists but is not called.

- **Location:** External trait impls after `verus!` block (exec, `pid.rs` lines 559-703)  
  **Description:** `Default`, `PartialEq`, `Ord`, `Debug`, `From`, and `TryFrom` implementations remain outside the `verus!` block. These impls are still unverified; the new wrappers help but do not make the trait impls themselves verified.  
  **Status vs previous review:** **Not fixed.** Still an unverified coverage gap.

### Low
- **Location:** Public field `value` (exec, `pid.rs` lines 72-75)  
  **Description:** The field is still `pub`, which exposes an API not present in the original tuple struct. The added comment notes this, but it does not prevent proofs from depending on public field access.  
  **Status vs previous review:** **Not fixed.** Still an encapsulation mismatch.

- **Location:** New byte-range axiom (proof, `pid.proof.rs` lines 108-121)  
  **Description:** `axiom_from_ne_bytes_in_range` is a new `external_body` axiom that widens the trusted base for serialization. This is a reasonable assumption, but it is additional trust not previously present and should be explicitly tracked as part of the TCB.  
  **Status vs previous review:** **New issue.** Trusted base expanded.

## Positive Observations
- The added verified wrapper methods (`eq`, `lt`, `le`, `gt`, `ge`, `default_value`) provide clean specs for the behavioral core and are consistently used by the trait impls.
- The layout lemmas (`lemma_size_eq_i32`, `lemma_align_eq_i32`) and `assert_layout` are clearly documented and now exist; they just need to be invoked to close the gap.
- Documentation of trust boundaries is improved and more explicit in `pid.rs` and `pid.proof.rs`.

## Summary
The prover made partial progress (added layout proof and helper methods), but two of the prior medium issues remain: layout is not enforced and trait impls are still unverified. The public field mismatch also remains, and a new external-body axiom expands the trusted base. Verification is therefore still incomplete; the fixes are only partial.
