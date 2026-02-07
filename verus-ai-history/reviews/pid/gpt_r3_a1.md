# Review: pid (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessIdentifier` definition and constants (exec: `verus/split/kernel/pm/sys/pid.rs`).
  **Description:** The verified struct exposes the raw field as `pub` and makes `PARSE_ERROR_MESSAGE` public, whereas the original uses a private tuple field and a private constant. This changes the public API and weakens encapsulation (external code can access/modify the raw value directly and rely on the error string), which is not semantically equivalent to the original interface.
  **Suggested Fix:** Keep the field and parse message private (`pub(crate)` or private tuple struct), and provide spec access through `spec_value()`/`view()` for verification. If Verus requires field visibility, consider a `pub(crate)` field plus a comment explaining the restricted visibility, and keep `PARSE_ERROR_MESSAGE` non-public.

### Low
- **Location:** External trait implementations (exec: `verus/split/kernel/pm/sys/pid.rs` outside `verus!`).
  **Description:** `Default`, `PartialEq`, `Ord`, `Debug`, and `From/TryFrom` impls are unverified Rust code. While they delegate to verified methods, the trait bodies themselves are not checked by Verus, so coverage is not complete for the original trait methods.
  **Suggested Fix:** Keep the delegating impls but add explicit `#[verifier::external_body]` annotations or proof wrappers that state their postconditions, or document these trait impls as part of the trusted boundary for this module.

## Positive Observations
- All core conversion and comparison operations are specified with precise ensures clauses, and error paths are constrained to `ErrorCode::InvalidArgument` with the correct message.
- Byte-serialization assumptions are explicitly documented, and the trust boundary is clearly identified with dedicated axioms and wrapper lemmas.
- Spec/proof/exec split is clean and supports reuse, with proofs isolated in `pid.proof.rs` and specs in `pid.spec.rs`.

## Summary
The verification captures the key behavioral properties for PID conversions, ordering, and constants, with clear documentation of the byte-serialization trust boundary. The main gaps are API-level equivalence issues (public field/constant) and unverified trait impls. Addressing those would bring the module closer to full coverage and semantic parity with the original.
