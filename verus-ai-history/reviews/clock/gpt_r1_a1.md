# Review: clock (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Missing verified public APIs (exec/spec/proof)**: `timer_handler`, standalone `ticks()`, and `now()` from the original source are not present as verified functions. The Verus module only verifies `TimerTicks` methods and helper arithmetic, so the public OS-facing APIs are unverified and key side effects (VM pause check, context switch) are outside the verified surface. **Suggested Fix:** Add verified wrappers for `timer_handler`, `ticks`, and `now` in `clock.rs` with specs in `clock.spec.rs`, and connect them to the modeled `TimerTicks` state and relevant postconditions.

### Medium
- **Global state and atomic semantics not modeled (exec/spec)**: The original uses a `static TIMER_TICKS` with atomic loads/stores and `&self` mutation via interior mutability. The verified model uses an owned `TimerTicks` with `&mut self` and no global singleton, so there is no proof that the verified state corresponds to the runtime global or that `get()` returns a consistent snapshot. This weakens equivalence and hides possible torn reads. **Suggested Fix:** Introduce a global state model and an invariant capturing single-writer/atomic-read consistency (or explicitly assume and justify it), and specify `get()` to return a consistent pair.
- **`now()` specification incomplete (spec/proof)**: Only arithmetic helpers (`compute_seconds`, `compute_nanoseconds`) are verified. There is no spec that `now()` returns a `SystemTime` consistent with `ticks()` or that it is monotonic. **Suggested Fix:** Specify `now()` in terms of `spec_compute_seconds`/`spec_compute_nanoseconds` and prove the returned `SystemTime` matches the combined tick count and is non-decreasing across increments.
- **Timer frequency assumption not justified (spec/proof)**: `compute_nanoseconds` and `compute_seconds` require `timer_freq > 0`, but there is no proof that `pit::get_timer_frequency()` is non-zero or that the cfg fallback is always used correctly. **Suggested Fix:** Add a spec/lemma for the platform timer frequency guaranteeing `> 0`, or model the `#[cfg]` cases with explicit proof obligations.
- **Missing properties of `timer_handler` (spec/proof)**: The proof only states a weak lemma about `spec_next_ticks` being bounded; it does not prove that `timer_handler` increments exactly once, checks pause requests correctly, or attempts a context switch when the kernel is not running. **Suggested Fix:** Add a specification for `timer_handler` and prove its observable effects, or clearly mark these behaviors as trusted and isolate them behind explicit assumptions.

### Low
- **Encapsulation divergence (exec)**: Fields are `pub` in the verified model and `increment()` requires `&mut self`, which is stronger than the runtime API and weakens encapsulation. This is documented, but it still creates a gap between the verified interface and the runtime one. **Suggested Fix:** Use a private-field model with accessor specs (or a wrapper type) to preserve the original interface shape.

## Positive Observations
- The arithmetic proofs for split 64-bit ticks (major/minor) are detailed and consistent with the original `wrapping_add` behavior.
- The nanosecond-range proof correctly establishes `SystemTime::new()`’s precondition and avoids overflow in the arithmetic model.
- Spec/proof separation is clean: `clock.spec.rs` contains abstract functions and `clock.proof.rs` contains lemmas, with exec code focused on state transitions.

## Summary
The verification is strong for internal counter arithmetic but does not cover the actual public APIs (`timer_handler`, `ticks`, `now`) or the global/atomic runtime model, leaving key correctness, safety, and liveness properties unverified. Strengthen the surface by adding specs and proofs for the real APIs and by modeling or explicitly assuming the global/atomic behavior and timer frequency guarantees.
