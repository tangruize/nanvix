# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage/semantics gaps for core methods** (exec: `process_state.rs` — `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`, `get_pmio_stub`, `get_pmio_mut_stub`).
  - **Description:** These correspond to public/private methods in the original module but are only `external_body` frame stubs with no behavioral specs. This leaves major portions of `ProcessState` unverified and fails the “all functions have verified versions” criterion in any semantic sense.
  - **Suggested Fix:** Replace stubs with verified wrappers that (a) model their ghost effects (mailbox queue, event list, mmio list, pmio operations, vmem side conditions) and (b) specify error codes/return values. If full modeling is out of scope, at least add precise pre/postconditions and a proof layer that links runtime behavior to ghost state.

- **API/equivalence mismatches** (exec: `process_state.rs` — `new`, `get_mutex`, `get_cond`, `remove_pmio`).
  - **Description:** Verified signatures diverge from the original: `new` drops the `Vmem` parameter; `get_mutex`/`get_cond` return `Ghost<nat>` instead of `Mutex`/`Condvar`; `remove_pmio` returns `Result<(), Error>` instead of the removed `AnyIoPort`. These mismatches mean the verified model is not semantically equivalent to the real implementation and cannot be substituted without additional refinement proofs.
  - **Suggested Fix:** Introduce opaque ghost tokens for returned objects and model ownership/identity (e.g., `Ghost<MutexHandle>`), keep the original signature shapes, and relate them to ghost maps/seqs. For `new`, include an abstract `vmem` token or a ghost field to represent initialization.

### Medium
- **Oracle parameters bypass runtime computation** (exec: `process_state.rs` — `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, `remove_pmio`).
  - **Description:** `already_present`, `contains`, `ref_count_at_threshold`, `found`, and `found_idx` are supplied by the caller and only constrained by preconditions. This shifts correctness to unverified call sites and assumes a faithful BTreeMap/LinkedList ↔ ghost relation without proof.
  - **Suggested Fix:** Add a ghost-to-runtime coupling invariant (e.g., a verified wrapper around `BTreeMap`/`LinkedList` or a refinement lemma) so the oracle values are *derived* and proven correct rather than assumed.

- **Hard-coded capacity constants may drift** (spec/exec: `process_state.spec.rs` — `MUTEX_MAX`, `COND_MAX`; exec: `MUTEX_MAX_EXEC`, `COND_MAX_EXEC`).
  - **Description:** The values are fixed at 32 with a comment referencing `kernel_config.toml`. If config changes, the verification silently becomes unsound.
  - **Suggested Fix:** Reuse a shared constant exported from the config layer or add a build-time proof/assumption tying the constant to the config source.

- **PMIO model lacks range/type constraints** (spec: `process_state.spec.rs` — `spec_pmio_ports`, `wf`).
  - **Description:** Ports are modeled as unconstrained `int` values, while the original uses `u16`. This permits negative/out-of-range ports in the verified model, weakening safety guarantees.
  - **Suggested Fix:** Add an invariant `forall p in pmio_ports: 0 <= p <= 0xFFFF` (or a ghost type wrapper) and ensure `add_pmio`/`remove_pmio` preserve it.

### Low
- **No liveness properties for mailbox/event handling** (spec/exec: stubs for `post_message`/`receive_message`, `add_event`/`remove_event`).
  - **Description:** The verification does not express or prove any delivery or progress guarantees. While not always required, this omits a key class of properties for process state.
  - **Suggested Fix:** If liveness is in scope, model a ghost message queue/event set and prove eventual delivery/removal properties under scheduling assumptions.

## Positive Observations
- Strong safety coverage for mutex/condvar capacity, ref-count semantics, and PID immutability.
- Well-structured spec/proof split with clear `wf()` invariant and auxiliary lemmas.
- PMIO first-occurrence semantics are explicitly modeled, matching `LinkedList::position` + `remove` behavior.

## Summary
The verification captures the core protocol for mutex/condvar management and PMIO list updates, but large portions of `ProcessState` are abstracted away via `external_body` stubs and API-shape changes, limiting equivalence and coverage. Tighten the model around omitted fields (events/mailbox/mmio/vmem) and remove oracle parameters by refining runtime data structures, then align signatures to the original to recover full semantic coverage.
