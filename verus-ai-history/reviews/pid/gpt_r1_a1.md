# Review: pid (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Public field breaks API equivalence** (exec: `ProcessIdentifier` in `verus/split/kernel/pm/sys/pid.rs`).
  - **Description:** The verified type exposes `pub value: i32`, while the original tuple field is private. This changes the public API and allows direct external access/mutation that the original code disallowed, so the verified module is not strictly semantically equivalent.
  - **Suggested Fix:** Make the field private (match the original), and keep spec access via `spec_value()`/`view()` in the same module. If external spec access is needed, provide `pub open spec fn` accessors rather than a public field.

- **Untrusted byte serialization assumptions** (exec/spec/proof: `to_ne_bytes`, `from_ne_bytes`, `axiom_*` in `pid.rs` and `pid.proof.rs`).
  - **Description:** Byte conversions are marked `external_body` with uninterpreted specs and round‑trip axioms. This is an explicit trust boundary in a core module and means the verification depends on unproven assumptions about Rust’s byte layout semantics.
  - **Suggested Fix:** Isolate these axioms in a small trusted module with clear justification, or replace with a verified bit‑level model if available. At minimum, document the trust boundary in module-level safety notes and avoid reusing these axioms outside this module.

- **Error content not specified** (exec/spec: `try_into_*`, `try_from_*` in `pid.rs`).
  - **Description:** Specs only distinguish Ok/Err but do not assert the `ErrorCode::InvalidArgument` and message used on failures. This weakens guarantees compared to the original implementation where error code/message are part of behavior.
  - **Suggested Fix:** Strengthen ensures to specify the error code and message in the Err case (assuming `Error` has spec fields or accessors).

### Low
- **Layout assertions missing** (exec: `pid.rs`).
  - **Description:** Original code includes size/alignment static asserts; the verified code relies on `#[repr(C)]` only. This omits an explicit check that layout matches the original ABI guarantees.
  - **Suggested Fix:** Add static assertions (or a proof lemma) for size/alignment to mirror the original guarantees.

## Positive Observations
- All original conversions and constants are represented, and verification passes for the module.
- Specs for conversions capture the key range conditions for signed/unsigned conversions.
- Spec/proof/exec separation is clean; proofs are isolated in `pid.proof.rs`.

## Summary
The verification covers the functional surface of PID conversions and constants, but there are notable soundness and equivalence gaps: the public field changes the API, and byte serialization relies on trusted axioms. Strengthening error specs and reinstating ABI layout checks would improve fidelity. Overall verification is solid but not airtight.
