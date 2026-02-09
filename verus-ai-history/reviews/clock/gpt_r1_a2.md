# Review: clock (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Missing verified public APIs (exec/spec/proof)** (verus/split/kernel/pm/clock.rs): The verified module still lacks exec/spec/proof for the original `timer_handler`, standalone `ticks()`, and standalone `now()` functions, so key OS-facing behaviors remain unverified. The new `TimerTicks::now(&self, timer_freq)` is not a drop-in for the original `now()` and does not model the global singleton or the HAL timer frequency selection. **Suggested Fix:** Add verified wrappers for `timer_handler`, `ticks`, and `now` that mirror the original signatures and wire them to the modeled counter and timer frequency assumptions.

### Medium
- **Global singleton/atomic snapshot still unmodeled** (verus/split/kernel/pm/clock.spec.rs: `spec_get_consistent`, verus/split/kernel/pm/clock.rs: `get()`): The new spec asserts that the returned pair recombines to `spec_ticks()`, but this does not address the atomic two-load snapshot or relate the model to the `static TIMER_TICKS`. The proof still assumes a consistent snapshot without a global invariant or explicit atomic model. **Suggested Fix:** Model the global singleton and add an invariant capturing single-writer + atomic-read consistency, or explicitly state and isolate the assumption with a dedicated trust boundary used by `get()`.
- **`now()` equivalence remains partial** (verus/split/kernel/pm/clock.rs: `TimerTicks::now`, verus/split/kernel/pm/clock.spec.rs: `spec_now`): The composed now returns a `(u64, u32)` pair and never constructs a `SystemTime`, so equivalence to the original `now()` (which returns `SystemTime`) is still unproven. There is also no verified path that selects `timer_freq` via PIT or fallback. **Suggested Fix:** Add a verified wrapper for the original `now()` that uses the correct `timer_freq` source and models `SystemTime::new()` (via a spec or external_body) with proven preconditions.
- **Timer frequency “fix” is vacuous and introduces unjustified `external_body`** (verus/split/kernel/pm/clock.proof.rs: `axiom_pit_timer_freq_valid`): The new axiom uses `external_body` but only proves the tautology `freq>0 ==> freq>0` because `spec_platform_timer_freq_valid` is defined as `freq > 0`. It does not connect to `pit::get_timer_frequency()` or justify the precondition. **Suggested Fix:** Replace the axiom with a specification of `pit::get_timer_frequency()` (or a wrapper) whose postcondition guarantees `> 0`, and use that in `now()`.
- **timer_handler behavior still only assumed** (verus/split/kernel/pm/clock.spec.rs: `spec_timer_handler_effect`, verus/split/kernel/pm/clock.proof.rs: lemmas): The new specs/lemmas are documentation-level assumptions; there is still no verified exec function that enforces “exactly one increment per call” or ties the handler to the model. **Suggested Fix:** Add an exec-level `timer_handler` model with a spec that directly invokes `increment()` once and explicitly lists trusted HAL side effects.

### Low
- **Encapsulation divergence remains** (verus/split/kernel/pm/clock.rs: `TimerTicks`): Fields remain `pub` and mutation uses `&mut self`, which is stronger than the original `&self` + atomics. This weakens interface equivalence and hides potential misuse. **Suggested Fix:** Use a private-field model with spec accessors or a wrapper type that preserves the original API shape.

## Positive Observations
- Added composed `now()` specs and proofs (seconds consistency and monotonicity) improve internal arithmetic coverage.
- `spec_get_consistent` and related comments clarify the intended snapshot property, even though the atomic model is still assumed.
- The proof structure remains cleanly split between exec/spec/proof files.

## Summary
Several improvements were added, but the core coverage and equivalence gaps remain: the original public APIs (`timer_handler`, standalone `ticks`, standalone `now`) are still not verified, and the global/atomic model and timer frequency justification are still assumptions. The new PIT-frequency axiom is vacuous and uses `external_body` without establishing a real link to the HAL, so soundness is not materially improved.
