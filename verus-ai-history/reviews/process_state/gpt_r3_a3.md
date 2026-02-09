# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage/soundness gaps remain for core methods** (exec: `process_state.rs` — `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`, `get_pmio_stub`, `get_pmio_mut_stub`).
  - **Description:** These are still `external_body` frame stubs with no behavioral specs, so large portions of `ProcessState` remain unverified and the core module’s correctness is not captured beyond non-interference.
  - **Suggested Fix:** Replace stubs with verified wrappers that model mailbox/event/mmio/pmio/vmem behavior in ghost state and specify return values and error conditions, or add refinement proofs to a verified abstract model.

- **API/semantic mismatches persist** (exec: `process_state.rs` — `new`, `get_mutex`, `get_cond`, `remove_pmio`, `read_pmio_stub`, `write_pmio_stub`).
  - **Description:** Signatures still diverge from the original (e.g., `new` lacks `Vmem`, `get_mutex/get_cond` return `Ghost<nat>`, `remove_pmio` returns `Result<(), Error>` instead of the removed port, pmio read/write stubs drop parameters). This breaks semantic equivalence without refinement.
  - **Suggested Fix:** Preserve original signature shapes using opaque ghost tokens for returned objects and a ghost `vmem` token; relate them to the ghost maps/seqs with refinement lemmas.

### Medium
- **Oracle parameters still shift correctness to callers** (exec: `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, `remove_pmio`).
  - **Description:** `already_present`, `contains`, `ref_count_at_threshold`, `found`, and `found_idx` remain externally supplied and only constrained by preconditions. There is still no verified coupling between runtime structures and ghost state.
  - **Suggested Fix:** Add a verified wrapper or refinement invariant tying runtime `BTreeMap`/`LinkedList` state to ghost maps/sequences so oracle values are derived and proven.

- **Capacity constants still hard-coded** (spec: `MUTEX_MAX`, `COND_MAX`; exec: `MUTEX_MAX_EXEC`, `COND_MAX_EXEC`).
  - **Description:** Values are still fixed at 32 with only comments referencing config. If configuration changes, the proof silently diverges.
  - **Suggested Fix:** Use a shared constant from the config layer or add a proof/assumption that binds the spec constants to the configuration source.

### Low
- **Mailbox/event behavior and liveness remain unspecified** (exec: stubs for `post_message`, `receive_message`, `add_event`, `remove_event`).
  - **Description:** The verification still does not model queue/event semantics or progress properties beyond frame conditions.
  - **Suggested Fix:** Add ghost state for mailbox/events with enqueue/dequeue/remove specs; optionally add liveness assumptions if required by system guarantees.

## Positive Observations
- PMIO port range constraints are now enforced in `wf()`, `add_pmio`, and `remove_pmio`, fixing the earlier range gap.
- Core mutex/condvar safety properties, capacity checks, and PID immutability remain well specified and verified.
- Verification run passes for the module (47 verified, 0 errors).

## Summary
The PMIO range issue is now addressed, but the major coverage, equivalence, and oracle-parameter soundness gaps remain. The verification is still a protocol model with large unmodeled behaviors and signature mismatches, so it is not yet complete or fully sound with respect to the original module.
