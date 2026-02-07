# Review: mutex (gemini-3-pro-preview) - Round 2

## Grade: B-

## Status: Failed

## Summary
The prover has significantly improved the documentation, providing excellent transparency regarding the limitations of the verification model (sequential, non-blocking, specification-only). This effectively addresses the "Concurrency Model Mismatch" and "Liveness" issues by explicitly defining them as out-of-scope.

However, the "Token Construction Trust Assumption" remains a critical, unaddressed issue. The prover documented it as a known vulnerability (Trust Assumption T3) rather than fixing it, despite a straightforward technical solution being available.

## Resolved Issues (via Documentation)
- **Concurrency Model Mismatch:** The header now clearly states this is a "sequential state machine correctness" proof and explicitly excludes concurrency/atomicity. While the model remains sequential (`&mut self`), the "not a runtime replacement" disclaimer is sufficient context.
- **Unrealistic Liveness Precondition:** The limitations of the `spec_is_unlocked()` precondition are now clearly documented as modeling only the "instant-success case."

## Remaining Issues

### High
- **Avoidable Trust Assumption (Token Forgery)**
  - **Location:** `mutex.spec.rs` (`MutexToken`)
  - **Status:** **Rejected / Not Fixed**
  - **Description:** The `MutexToken` struct still has a `pub ghost view` field. The prover added "Trust Assumption T3" stating that "external code could theoretically construct a token... mutual exclusion guarantee relies on the assumption that tokens are created solely via the module's API."
  - **Critique:** This trust assumption is unnecessary and unsafe. In Verus/Rust, you can restrict field visibility to the module. By making the `view` field private and providing a `pub open spec fn view(&self) -> MutexView` getter, you can mechanically enforce that only the `mutex` module can construct valid tokens, removing T3 entirely. Documenting a vulnerability is not a substitute for fixing it when the fix is simple encapsulation.
  - **Required Fix:** Make `MutexToken.view` private and add a public spec getter.

### Low
- **Global ID Uniqueness (T1)**
  - **Location:** `mutex.rs` (`new`)
  - **Description:** The requirement for callers to provide a unique `id` is a significant burden on the client and a fragile verification guarantee. While harder to fix than the Token issue, it remains a weak point in the safety interface.
