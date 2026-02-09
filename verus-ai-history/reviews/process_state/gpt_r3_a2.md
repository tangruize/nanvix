# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage/soundness gaps remain for core methods** (exec: `process_state.rs` — `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`, `get_pmio_stub`, `get_pmio_mut_stub`).
  - **Description:** These still rely on `external_body` stubs with only frame conditions (or `ensures true`), so there is no verified behavioral specification for large parts of the module. This does not satisfy coverage of the original public/private functions in a semantic sense and leaves the core module’s behavior unverified.
  - **Suggested Fix:** Replace stubs with verified wrappers that model mailbox/event/mmio/pmio/vmem behavior in ghost state and specify return values and error conditions; or supply refinement proofs that connect these operations to a verified abstract model.

- **API/semantic mismatches persist** (exec: `process_state.rs` — `new`, `get_mutex`, `get_cond`, `remove_pmio`).
  - **Description:** Signatures still diverge from the original: `new` drops the `Vmem` parameter, `get_mutex`/`get_cond` return `Ghost<nat>` instead of the mutex/condvar object, and `remove_pmio` returns `Result<(), Error>` instead of the removed port. This remains a non-equivalence gap with no refinement proof.
  - **Suggested Fix:** Keep original signature shapes using opaque ghost tokens for returned objects (e.g., handles representing `Arc<Mutex>`/`Arc<Condvar>`/`AnyIoPort`) and relate them to the ghost maps/sequences; include an abstract `vmem` token for `new`.

### Medium
- **Oracle parameters still shift correctness to callers** (exec: `process_state.rs` — `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, `remove_pmio`).
  - **Description:** `already_present`, `contains`, `ref_count_at_threshold`, `found`, and `found_idx` remain externally provided and only constrained by preconditions. There is still no verified coupling between runtime data structures and ghost state.
  - **Suggested Fix:** Introduce a verified wrapper or refinement invariant tying runtime `BTreeMap`/`LinkedList` state to ghost maps/sequences so oracle values are derived and proven, not assumed.

- **Capacity constants still hard-coded** (spec: `process_state.spec.rs` — `MUTEX_MAX`, `COND_MAX`; exec: `MUTEX_MAX_EXEC`, `COND_MAX_EXEC`).
  - **Description:** Values are still fixed at 32 and only referenced by comment. If configuration changes, the proof silently diverges.
  - **Suggested Fix:** Use a shared constant from the config layer or add a proof/assumption that binds the spec constants to the configuration source.

### Low
- **PMIO input range not enforced uniformly** (exec/spec: `remove_pmio`, `read_pmio_stub`, `write_pmio_stub`).
  - **Description:** `wf()` now enforces port values are within `0..=0xFFFF` and `add_pmio` requires the range, but `remove_pmio` (and pmio read/write stubs) accept arbitrary `int` inputs. This is weaker than the original `u16` signature and allows out-of-range calls in the model.
  - **Suggested Fix:** Add `0 <= port_number <= 0xFFFF` preconditions for all PMIO APIs or model the port number as a ghost `u16`-like type.

- **Mailbox/event liveness and behavior remain unspecified** (exec: stubs for `post_message`, `receive_message`, `add_event`, `remove_event`).
  - **Description:** The verification still does not model message queue or event ownership semantics beyond frame conditions, so safety/liveness properties for these operations are not proven.
  - **Suggested Fix:** Add ghost state for mailbox/event sets and specify enqueue/dequeue/remove behavior; optionally add liveness assumptions if required.

## Positive Observations
- The PMIO range invariant has been added to `wf()` and `add_pmio` now enforces valid `u16` ranges; proof lemmas updated accordingly.
- Core mutex/condvar safety properties, capacity checks, and PID immutability remain well specified and verified.
- Verification run passes for the module (47 verified, 0 errors).

## Summary
One prior issue (PMIO range constraints) is now properly addressed, but the major coverage, equivalence, and oracle-parameter soundness gaps remain. The verification is still a protocol model with large unmodeled behaviors and signature mismatches, so it is not yet complete or fully sound with respect to the original module.
