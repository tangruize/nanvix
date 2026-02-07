# Review: pid (gpt-5.1-codex-max)

## Grade: B

## Issues Found

### High
- FFI/layout guarantees still incomplete (exec/spec/proof: verus/split/kernel/pm/sys/pid.rs): `#[repr(C)]` was restored, but there are still no static size/alignment assertions or proof obligations tying the verified type to the original FFI layout. The earlier concern about layout guarantees being unverified remains; please add the size/align checks or explicit invariants to match the original contract.
- Abstraction leak on PID value (exec/spec: pid.rs/pid.spec.rs): The `value` field stays `pub`, so callers can bypass the verified constructors and invariants just as before. The original tuple struct kept the field private; please restore privacy (or restrict to `pub(crate)` with accessor proofs) to prevent bypassing the spec’d API.

### Medium
- Byte round-trip still assumed (proof: pid.proof.rs): `to_ne_bytes`/`from_ne_bytes` correctness continues to rely on external-body axioms (`axiom_byte_roundtrip` / `axiom_bytes_roundtrip`) with no supporting proof or linkage to trusted specs, leaving serialization soundness as an assumption. Replace these axioms with proved lemmas or connect to a trusted byte-representation spec.
- Trait API verification gap (exec: pid.rs): The `Default`, `Debug`, `From`/`TryFrom`, and comparison trait impls are reintroduced, but they sit outside the verified block and have no proofs tying them to the spec’d semantics or range checks. Either verify these trait behaviors directly or wrap them through the already-verified methods with specs that ensure parity.

## Positive Observations
- `#[repr(C)]` is now present, and the original trait surface has been reinstated, improving API parity with the kernel type.
- Conversion methods continue to carry range checks with clear ensures clauses.

## Summary
Good progress on restoring API parity, but key guarantees remain unverified: layout soundness lacks size/align checks, the raw field is still public, and byte round-trips and trait behaviors are assumed rather than proved. Tighten encapsulation, replace axioms with proofs, and add layout assertions to reach full coverage.
