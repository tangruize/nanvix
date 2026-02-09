# Review: clock (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Original public APIs still not fully verified** (verus/split/kernel/pm/clock.rs): The new `timer_handler_model`, `standalone_ticks`, and `standalone_now` do not match the original signatures or global singleton behavior (`static TIMER_TICKS`, `InterruptNumber`, `unsafe`, `SystemTime` return). These are models, not verified versions of the real public APIs, so coverage and equivalence gaps remain. **Suggested Fix:** Add exec-level wrappers with the original signatures that access a modeled singleton and, for `now()`, return a modeled `SystemTime` value (or an `external_body` wrapper with verified preconditions).

### Medium
- **Global singleton/atomic snapshot still assumed** (verus/split/kernel/pm/clock.spec.rs: `spec_get_consistent`): The spec now documents assumptions about atomic snapshot consistency, but there is still no modeled global state or invariant tying the Verus `TimerTicks` to the runtime `static TIMER_TICKS`. This leaves equivalence to the real implementation unproven. **Suggested Fix:** Add a modeled singleton with an invariant reflecting the single-writer assumption and explicitly use it in `standalone_*` functions.
- **`now()` equivalence remains partial** (verus/split/kernel/pm/clock.rs: `standalone_now`): The standalone model returns a `(u64, u32)` pair and never constructs a `SystemTime`, so it does not prove that the real `now()` returns the same `SystemTime` value. The spec only models the `SystemTime::new()` precondition, not its full semantics. **Suggested Fix:** Introduce a modeled `SystemTime` (or a wrapper with `external_body`) and prove that `standalone_now()` produces the same value as the runtime `now()`.
- **Timer frequency trust boundary still disconnected** (verus/split/kernel/pm/clock.proof.rs: `axiom_pit_timer_freq_valid`): The new axiom asserts `freq > 0` but is not linked to `pit::get_timer_frequency()` or the `#[cfg]` selection in the real code. This is still a free assumption and does not justify the precondition for the actual `now()` implementation. **Suggested Fix:** Provide a spec for `pit::get_timer_frequency()` or a wrapper function whose postcondition guarantees `> 0`, and use that in a verified `now()` wrapper.

### Low
- **Encapsulation divergence persists** (verus/split/kernel/pm/clock.rs: `TimerTicks`): Fields remain `pub` and mutation uses `&mut self`, which diverges from the original `&self` + atomics interface. **Suggested Fix:** Use private fields and spec accessors (or a wrapper) to preserve the runtime API shape.

## Positive Observations
- Added exec-level models and specs improve internal arithmetic coverage, including now() seconds consistency and SystemTime precondition proof.
- The timer_handler model now explicitly proves a single increment, which is a step toward behavioral coverage.
- Documentation clearly enumerates trust boundaries and assumptions.

## Summary
Some improvements were added, but core equivalence and coverage gaps remain: the real public APIs and global singleton behavior are still not verified, and the PIT timer frequency assumption is not tied to the actual HAL call. Verification is stronger internally but still incomplete for OS-facing correctness.
