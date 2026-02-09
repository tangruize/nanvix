# Review: clock (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `TimerTicks::get` spec/exec (`verus/split/kernel/pm/clock.spec.rs`, `clock.rs`).
  **Description:** The verified model treats `get()` as returning a consistent snapshot and proves `spec_get_consistent` unconditionally, while the real code performs two separate atomic loads that can observe a torn state if an interrupt fires between loads. The only “assumption” in the spec (`spec_no_concurrent_writer_assumption`) is a `true`-returning placeholder, so the proof does not actually require the interrupt-disabled/single-writer precondition that the real implementation depends on. This makes the spec stronger than the implementation and can justify properties (e.g., monotonic seconds) that would be violated by a torn read.
  **Suggested Fix:** Encode the snapshot condition as a real precondition (e.g., `requires no_concurrent_writer()`), or model `get()` as potentially torn. Alternatively, change the exec code to use a loop/seqlock or double-read scheme that guarantees a consistent snapshot, then verify that stronger behavior.

### Medium
- **Location:** `standalone_ticks`, `standalone_now`, and `timer_handler_model` (`verus/split/kernel/pm/clock.rs`).
  **Description:** The verification abstracts away the `static TIMER_TICKS` singleton and the actual `timer_handler` integration. The models operate on arbitrary `TimerTicks` instances and do not establish a module-level invariant that the global singleton is the only writer and that the handler is invoked once per interrupt. This leaves a gap between the verified model and the concrete kernel module wiring.
  **Suggested Fix:** Introduce a ghost model of the `TIMER_TICKS` singleton and a module invariant tying `timer_handler`, `ticks()`, and `now()` to that global state (including the single-writer / interrupt serialization assumption). This will close the equivalence gap at the module boundary.

### Low
- **Location:** `axiom_pit_timer_freq_valid` (`verus/split/kernel/pm/clock.proof.rs`).
  **Description:** The PIT frequency positivity is modeled via an `external_body` axiom but is not linked to the concrete `pit::get_timer_frequency()` call in the original `now()` implementation. This is a trust boundary that is currently documentation-only and could be made more explicit.
  **Suggested Fix:** Wrap the HAL timer frequency function in an external spec or add a small wrapper with a verified postcondition (`> 0`) used directly in the `now()` model.

## Positive Observations
- The arithmetic model for the split counter is strong: wrapping behavior, u64 combination, and non-overflow properties are all proved with clear lemmas.
- The `now()` proof explicitly guarantees `SystemTime::new()` preconditions and eliminates the `unreachable!()` path, matching the source behavior.
- Trust boundaries are clearly documented, and the spec/proof separation is clean and readable.

## Summary
The verification is thorough for counter arithmetic and `now()` safety, but it assumes a consistent snapshot for `get()` without formally tying that to the kernel’s interrupt and single-writer discipline. Strengthening the concurrency precondition or the implementation itself would close the main soundness/equivalence gap and raise confidence in the verified behavior at the module level.
