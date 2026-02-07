# Review: pid (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessIdentifier::assert_layout` (proof, `pid.proof.rs` lines 155-171)  
  **Description:** The layout proof is still never invoked. A repo-wide search only finds the definition, not a call site, so the size/alignment assertions remain unexercised and the ABI/layout requirement is not enforced.  
  **Status vs previous review:** **Not fixed.**

- **Location:** External trait impls after `verus!` block (exec, `pid.rs` lines 559-703)  
  **Description:** `Default`, `PartialEq`, `Ord`, `Debug`, `From`, and `TryFrom` implementations remain outside the `verus!` block, so the trait behavior is still unverified (only the wrapper methods are verified).  
  **Status vs previous review:** **Not fixed.**

### Low
- **Location:** Public field `value` (exec, `pid.rs` lines 72-75)  
  **Description:** The field is still `pub`, exposing an API not present in the original tuple struct. The note acknowledges it, but it does not prevent proofs from depending on direct field access.  
  **Status vs previous review:** **Not fixed.**

- **Location:** Byte-range axiom (proof, `pid.proof.rs` lines 108-121)  
  **Description:** `axiom_from_ne_bytes_in_range` remains an `external_body` axiom, extending the trusted base for serialization. This should be explicitly tracked as part of the TCB.  
  **Status vs previous review:** **Not fixed.**

## Positive Observations
- The verified wrapper methods still give precise specs for conversions and comparisons.
- Layout lemmas and `assert_layout` remain well-documented; only the call site is missing.
- Trust boundary documentation remains clear and explicit.

## Summary
No evidence that the prior issues were fixed: layout is still unenforced, trait impls remain unverified, the public field mismatch persists, and the extra serialization axiom remains. Verification is therefore still incomplete and the changes do not fully address the previous review.
