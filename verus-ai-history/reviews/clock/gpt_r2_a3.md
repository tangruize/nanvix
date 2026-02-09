# Review: clock (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `standalone_ticks` (`verus/split/kernel/pm/clock.rs`).
  **Description:** The original `ticks()` reads via `TIMER_TICKS.get()` (two atomic loads), which requires the same no-concurrent-writer/interrupt-serialization assumption as `get()`. The verified `standalone_ticks` model bypasses `get()` and reads fields directly with no precondition, so it can prove a stronger property than the real implementation when torn reads are possible.
  **Suggested Fix:** Either model `standalone_ticks` using `get()` (and inherit its precondition) or add `requires spec_no_concurrent_writer_assumption()` to `standalone_ticks` to match the runtime dependency.

- **Location:** `timer_handler_model`, `standalone_ticks`, `standalone_now` (`verus/split/kernel/pm/clock.rs`).
  **Description:** The verification still does not model the `static TIMER_TICKS` singleton or establish a module-level invariant tying the global state to the public APIs and the real `timer_handler`. This leaves the equivalence between the verified model and the concrete kernel wiring incomplete.
  **Suggested Fix:** Introduce a ghost model of the global singleton and a module invariant showing that `timer_handler` is the sole writer and `ticks()`/`now()` read that singleton, then use that invariant in the standalone models.

### Low
- **Location:** `axiom_pit_timer_freq_valid` / `standalone_now` (`verus/split/kernel/pm/clock.proof.rs`, `clock.rs`).
  **Description:** The PIT frequency axiom is documented as modeling the HAL call, but `standalone_now` still takes an unconstrained `timer_freq` parameter; there is no verified wrapper tying it to `pit::get_timer_frequency()`.
  **Suggested Fix:** Provide a small verified wrapper that calls the HAL function and returns a `u32` with `> 0`, then use it in the `now()` model.

## Positive Observations
- The snapshot consistency assumption is now a real precondition of `get()`/`now()`, which fixes the prior unsound “assumed true” path.
- Arithmetic proofs for increment, tick composition, and `now()` safety remain strong and well-separated from exec code.

## Summary
The concurrency assumption is now enforced at the API boundary, which is a real fix. Remaining gaps are mostly about module-level equivalence: `standalone_ticks` still bypasses the snapshot precondition, and the global singleton wiring is not modeled.
