# Review: clock (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `TimerTicks::get` + `spec_no_concurrent_writer_assumption`/`axiom_no_concurrent_writer` (`verus/split/kernel/pm/clock.rs`, `clock.spec.rs`, `clock.proof.rs`).
  **Description:** The previous snapshot-consistency gap is only partially addressed. The assumption is now opaque, but `get()` unconditionally invokes the external-body axiom and returns the predicate to callers. This still lets any call site obtain snapshot consistency without proving interrupts are disabled or that no concurrent writer exists. As a result, the verified monotonicity/consistency properties can still be stronger than the real implementation when the system invariant is not enforced.
  **Suggested Fix:** Make the no-concurrent-writer predicate an explicit precondition of `get()` (and hence `now()`/`ticks()`), or model torn reads in the exec/spec. Alternatively, prove at the module boundary that all call sites run under the interrupt serialization invariant and only then allow the axiom to be used.

### Medium
- **Location:** `timer_handler_model`, `standalone_ticks`, `standalone_now` (`verus/split/kernel/pm/clock.rs`).
  **Description:** The verification still does not model the `static TIMER_TICKS` singleton or establish a module-level invariant tying the global state to the public `ticks()`/`now()` functions and the real `timer_handler`. This leaves the equivalence between the verified model and the concrete kernel wiring incomplete.
  **Suggested Fix:** Introduce a ghost model of the global singleton and a module invariant showing that `timer_handler` is the sole writer and `ticks()`/`now()` read that singleton, then use that invariant in the standalone models.

### Low
- **Location:** `axiom_pit_timer_freq_valid` / `standalone_now` (`verus/split/kernel/pm/clock.proof.rs`, `clock.rs`).
  **Description:** The PIT frequency axiom is documented as modeling the HAL call, but it is still not wired into the `now()` model; `timer_freq` is an unconstrained parameter and no proof links it to `pit::get_timer_frequency()`.
  **Suggested Fix:** Provide a small verified wrapper that calls the HAL function and returns a `u32` with `> 0`, then use it in the `now()` model.

## Positive Observations
- The snapshot consistency assumption is now explicitly modeled as an opaque predicate with a clear trust boundary, improving auditability.
- The arithmetic proofs for increment, ticks composition, and `now()` safety remain strong and cleanly separated from exec code.

## Summary
The update improves documentation and makes the concurrency assumption explicit, but the core soundness gap remains because the axiom is granted unconditionally and the global singleton is still not modeled. The verification is solid for arithmetic, yet still incomplete for module-level equivalence and concurrency assumptions.
