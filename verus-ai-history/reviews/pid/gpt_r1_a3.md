# Review: pid (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Public field still breaks API equivalence** (exec: `ProcessIdentifier` in `verus/split/kernel/pm/sys/pid.rs`).
  - **Status:** Not fixed. The field remains `pub`, so external code can read/mutate it directly.
  - **Why it matters:** This diverges from the original private field and changes the public API/encapsulation guarantees.

- **Byte serialization remains a trusted axiom in a core module** (exec/spec/proof: `to_ne_bytes`, `from_ne_bytes`, `axiom_*`).
  - **Status:** Not fixed. The axioms are still in-module and remain unverified assumptions.
  - **Why it matters:** Verification depends on unproven byte-level semantics, and the trust boundary is not isolated.

### Medium
- **Layout checks still not enforced** (proof: `assert_layout` in `pid.proof.rs`).
  - **Status:** Not fixed. A proof wrapper was added, but it is never invoked, so size/alignment are not actually asserted like the original static checks.
  - **Why it matters:** ABI/layout equivalence is not guaranteed unless `assert_layout` is exercised.

### Low
- **New public constant expands the API** (exec: `PARSE_ERROR_MESSAGE` is now `pub`).
  - **Status:** New issue. This constant was previously private and is now part of the public surface.
  - **Why it matters:** It further diverges from the original API and exposes an implementation detail.

## Positive Observations
- Error `reason` is now specified in Err ensures for all conversions, matching the concrete behavior.

## Summary
The update fixed the underspecified error reason, but the two key soundness/equivalence issues remain: public field exposure and trusted byte-serialization axioms. Layout assertions are still not enforced because the new wrapper proof is never called, and a new public constant further expands the API surface. Verification is improved but still not fully equivalent or airtight.
