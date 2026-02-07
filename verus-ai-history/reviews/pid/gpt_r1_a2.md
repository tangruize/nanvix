# Review: pid (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Public field still breaks API equivalence** (exec: `ProcessIdentifier` in `verus/split/kernel/pm/sys/pid.rs`).
  - **Status:** Not fixed. The field remains `pub` with a comment explaining the choice, but this still changes the public API compared to the original private tuple field.
  - **Why it matters:** External code can mutate/read the PID directly, which is not permitted in the original API.

- **Byte serialization remains a trusted axiom in a core module** (exec/spec/proof: `to_ne_bytes`, `from_ne_bytes`, `axiom_*`).
  - **Status:** Partially addressed via documentation, but the trust boundary is still inside the module and not isolated.
  - **Why it matters:** Verification still depends on unproven assumptions about byte layout/round-trip semantics.

### Medium
- **Err reason string not specified** (exec/spec: `try_into_*`, `try_from_*` in `pid.rs`).
  - **Status:** Partially fixed. Ensures now pin `ErrorCode::InvalidArgument`, but do not assert the `reason` field, even though `Error::new` specifies it and `Error` exposes `reason`.
  - **Why it matters:** Behavior is still weaker than the concrete implementation, which always returns the specific message.

- **Layout checks are not asserted, only declared** (proof: `lemma_size_eq_i32`, `lemma_align_eq_i32`).
  - **Status:** Partially fixed. Lemmas exist but are never invoked, so there is no actual assertion equivalent to the original static checks.
  - **Why it matters:** ABI/layout equivalence is not enforced unless a proof explicitly calls the lemmas.

### Low
- None.

## Positive Observations
- Error-code behavior is now specified in ensures clauses for conversion failures.
- Trust boundary around byte serialization is clearly documented at module level.

## Summary
Some improvements were made (error code specs, documentation), but two key issues remain unfixed: the public field still changes the API and byte serialization remains a trusted axiom inside the core module. Error reason strings are still underspecified, and the layout assertions are not actually enforced unless the new lemmas are invoked. Verification is improved but not complete or fully equivalent.
