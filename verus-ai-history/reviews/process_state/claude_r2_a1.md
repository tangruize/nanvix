# Review: process_state (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `get_mutex` (exec, line 337) / `get_cond` (exec, line 487)
  **Description:** The original `get_mutex` checks `self.mutexes.len() >= MUTEX_OPEN_MAX` *before* consulting the entry, so it rejects `get_mutex` for an **already-existing** key when at capacity. The verified code faithfully replicates this (`if self.mutex_count >= Self::MUTEX_MAX_EXEC()` runs first), so both reject a legitimate operation (returning an existing Arc clone that wouldn't grow the map). The spec's error postcondition says only `old(self).spec_mutexes_full()`, but does not assert `!already_present`. This means the spec *permits* an error even when the key exists — matching the original's overly-conservative check — but a tighter spec that allows success when `already_present` regardless of capacity would be more useful for callers and would catch this latent design bug. Same issue applies to `get_cond`.
  **Suggested Fix:** Add the postcondition `result is Err ==> !already_present` (or, if the intent is to exactly mirror the original's conservative check, add a comment explicitly documenting that this is a known over-approximation inherited from the source).

### Medium

- **Location:** `ProcessState::ghost_mutexes` / `ghost_conditions` (spec, proof)
  **Description:** The ghost ref-count model only *increments* on `get_mutex`/`get_cond` but never *decrements* when external code drops Arc clones. Over a sequence of get/put operations the ghost ref-count diverges upward from the actual `Arc::strong_count()`. The `ref_count_at_threshold` oracle parameter in `put_mutex`/`put_cond` bridges the gap at each call site, but this means the ghost map cannot be used to *predict* whether a future `put` will remove the entry — only to *observe* it. Trust assumption T3 documents this, but it limits compositional reasoning about sequences of operations.
  **Suggested Fix:** Consider adding a `dec_mutex_ref_count` ghost operation (called by callers when they drop their Arc) so the ghost map tracks the true count. Alternatively, add a proof lemma establishing that the ghost count is always ≥ the actual count (upper bound property).

- **Location:** `ProcessState::new` (exec, line 186)
  **Description:** The original constructor takes `(pid: ProcessIdentifier, vmem: Vmem)`, but the verified version takes only `(pid: ProcessIdentifier)`. The `Vmem` parameter is entirely elided, not even captured as a ghost token. While Vmem is an opaque boundary type, this means the verification cannot express properties like "the vmem in the constructed state is the one that was passed in." If a caller passes the wrong vmem or if vmem initialization has invariants, the verification model cannot catch it.
  **Suggested Fix:** Add a ghost `vmem_token: Ghost<int>` field (or similar opaque identifier) to the model so the spec can at least assert identity: `result.spec_vmem_token() == vmem_token@`. Low effort for some provenance tracking.

- **Location:** `receive_message_stub` (exec, line 774)
  **Description:** The original `receive_message` returns `Option<Message>`, consuming a message from the mailbox (a side-effecting operation). The stub declares `&mut self` and preserves all verified fields, which is correct, but the return type is `()` instead of `Result<(), Error>` or similar. This means the stub cannot be used to prove anything about message delivery — not even that a message was dequeued. All mailbox stubs (`post_message_stub`, `receive_message_stub`) treat the mailbox as fully opaque.
  **Suggested Fix:** If mailbox ordering or delivery guarantees matter for overall correctness, consider adding a ghost message queue to the model. If not, document that mailbox semantics are intentionally out of scope.

### Low

- **Location:** `ProcessRefMut` / `ProcessRef` (exec, lines 982–1028)
  **Description:** The original types are lifetime-parameterized enums (`ProcessRefMut<'a>`, `ProcessRef<'a>`) with five variants corresponding to process lifecycle states. The verified model drops lifetimes and variants entirely, modeling them as opaque unit structs with `external_body`. The `state_mut_stub` ensures only `true`, losing the connection between the enum variant and the returned `ProcessState` reference. This is acceptable given these are thin dispatch wrappers, but it means the verification cannot reason about which lifecycle state a process is in when accessing its state.
  **Suggested Fix:** If lifecycle-state-dependent properties become important (e.g., "only a Running process can perform I/O"), consider adding a ghost `ProcessLifecycleState` enum to the model.

- **Location:** `add_pmio` (exec, line 583)
  **Description:** The original `add_pmio` takes a full `AnyIoPort` object, but the verified version takes `Ghost<int>` (just the port number). The original `remove_pmio` returns `Result<AnyIoPort, Error>` (returning the removed port object), but the verified version returns `Result<(), Error>`. This means the verification cannot reason about port identity beyond the port number — e.g., it cannot prove that the port returned by `remove_pmio` is the same object that was added.
  **Suggested Fix:** Acceptable for protocol-level verification. If port identity matters, extend the ghost model to track `(port_number, port_token)` pairs.

- **Location:** `MUTEX_MAX` / `COND_MAX` spec constants (spec, lines 151–158)
  **Description:** The spec constants are hardcoded to `32usize` and documented as matching `build/kernel_config.toml`. Verified against the config file — values match (`mutex_open_max = 32`, `cond_open_max = 32`). However, if the config changes, the verification will silently become inconsistent. There is no automated link between the config and the spec constants.
  **Suggested Fix:** Add a comment in the spec noting the config file path and line number, or add a CI check that validates the constants match.

- **Location:** `put_mutex` / `put_cond` frame conditions on error path (exec)
  **Description:** On the error path (`result is Err`), the postconditions do not explicitly state `self.spec_pmio_ports() == old(self).spec_pmio_ports()` or `self.spec_capabilities_bits() == old(self).spec_capabilities_bits()`. These are implied by the fact that the error path returns early without modifying `self`, but making them explicit would strengthen the frame condition for callers.
  **Suggested Fix:** Add the missing frame conditions to the error postconditions for completeness.

## Positive Observations

- **Full function coverage:** All 26+ functions from the original (public and private, including `Debug::fmt`) have verified counterparts or frame-condition stubs. No function is silently omitted.
- **Clean verification:** 47 verified, 0 errors. No `assume` statements anywhere in the codebase.
- **No unjustified `external_body`:** All `external_body` annotations are on HAL/IPC/MM boundary type stubs. The six core state-management functions (`get_mutex`, `put_mutex`, `get_cond`, `put_cond`, `add_pmio`, `remove_pmio`) plus capabilities and construction are fully verified without `external_body`.
- **Thorough well-formedness invariant:** The `wf()` predicate covers ghost/runtime consistency (domain size = counter), capacity bounds (≤ MAX), capabilities well-formedness, and positive reference counts for all map entries. It is preserved by every operation.
- **PID immutability proven across all operations:** Dedicated proof lemmas establish that no operation modifies the process identifier.
- **Strong frame conditions:** Every mutating function specifies what it *doesn't* change (other collections, PID, capabilities) in addition to what it changes. This is essential for compositional verification.
- **Well-documented trust assumptions:** T1–T5 explicitly enumerate what the verification trusts without proof, making the trust boundary clear.
- **Good spec/proof/exec separation:** Spec functions and View type in `.spec.rs`, proof lemmas in `.proof.rs`, and executable code with inline postconditions in `.rs`. The proof file has meaningful lemmas (not just trivial assertions), including the inductive `lemma_pmio_min_index_helper` for first-occurrence well-ordering.
- **Oracle parameter pattern well-applied:** The use of `already_present`, `contains`, `ref_count_at_threshold`, and `found` as constrained oracle parameters is the standard Verus idiom for bridging ghost/exec boundaries. Preconditions properly tie them to ghost state.

## Summary

This is a solid protocol-level verification of the `ProcessState` module. The verification successfully captures the essential correctness properties of the state management protocol: PID immutability, bounded collection management with proper error codes, reference-count-based cleanup semantics for mutexes and condition variables, and I/O port tracking with first-occurrence removal semantics. All functions are covered, with appropriate use of frame-condition stubs for opaque HAL/IPC/MM boundary types.

The main limitation is that the ghost reference-count model is monotonically increasing (no decrement on Arc drop), which limits compositional reasoning about operation sequences. This is an inherent trade-off of the protocol-model approach and is well-documented. The other notable gap is the overly-conservative capacity check in `get_mutex`/`get_cond` (rejecting existing keys at capacity), which is faithfully modeled but could benefit from a tighter spec or explicit documentation.

The verification is clean (no `assume`, no unjustified `external_body`), well-structured, and well-documented. It would benefit from minor improvements to frame conditions on error paths and optional extensions for vmem provenance tracking and mailbox semantics, but these are enhancements rather than correctness issues.
