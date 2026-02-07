# Review: ready (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Missing verified `join_cond()`** (exec/spec/proof): The original `ReadyThread::join_cond()` is omitted in the Verus split, so coverage and equivalence are incomplete and any safety properties around join condition usage are unverified. **Suggested Fix:** Add an exec stub and spec/proof boundary model for `Condvar` (or an opaque token) and prove it preserves identity/wf, or explicitly justify exclusion with a module-level assumption list and link to where it is verified.
- **Unconstrained `thread_state_mut()` escape hatch** (exec): `thread_state_mut()` is `#[verifier::external]` with no enforced postconditions, so callers can violate `wf()`/identity and break proofs. **Suggested Fix:** Add a spec wrapper (or a verified forwarding API set) and strengthen documentation with a verified lemma or a trusted postcondition (via `external_body`) that preserves `spec_id` and `wf`, then audit/replace call sites.

### Medium
- **`run()` context pointer omitted** (exec/spec): The raw `*mut ContextInformation` return is dropped, so semantic equivalence for context handoff is not modeled. **Suggested Fix:** Model an abstract token or ghost handle for the context pointer and thread it through `run()` specs to capture the handoff, or explicitly scope it as an unverified HAL boundary in a tracked assumption section.

### Low
- **Admission-time semantics are weakly specified** (spec): `clock_now()` only ensures `>= 0`, so no ordering or freshness property is captured (e.g., admission time is “now”). **Suggested Fix:** If scheduling correctness depends on it, add a monotonicity/freshness axiom for `clock_now()` or model a ghost clock to relate admissions.

## Positive Observations
- All main transitions (`new`, `from_state`, `run`, `terminate`) are specified with clear identity, mutex, and drop-safety preservation.
- Boundary models for `RunningThread`/`ZombieThread` include explicit cross-module obligations, which helps future linking.
- Proofs cover key safety aspects of `run()` (interrupt clearing, mutex preservation) and `terminate()` (status/identity).

## Summary
Coverage is strong for core transitions, but missing `join_cond()` and the unconstrained `thread_state_mut()` escape hatch leave important gaps in equivalence and soundness. Addressing these boundary omissions and strengthening the context and time modeling would raise the verification to an A-range.
