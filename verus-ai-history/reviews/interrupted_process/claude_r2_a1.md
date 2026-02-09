# Review: interrupted_process (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

_None._

### High

- **Location:** `resume()` in exec (`interrupted.rs:268-374`) — thread state mutation not modeled
  - **Description:** In the original `InterruptedThread::resume()` (`src/kernel/src/pm/thread/interrupted.rs:113-116`), the thread's state is mutated via `self.state.set_interrupt_reason(self.reason)` before being converted to a `ReadyThread`. This stores the interrupt reason into the thread's state. The verification model treats `resume()` as purely ID-preserving and does not model the `interrupt_reason` field or its mutation. If downstream code relies on `interrupt_reason` being set (e.g., to determine why a thread was resumed), this property is not verified.
  - **Suggested Fix:** Add a ghost field `interrupt_reason: Ghost<Option<int>>` to the thread model, and ensure `resume()` postcondition asserts the interrupt reason is set. Alternatively, document this as an explicit trust boundary if `interrupt_reason` is never checked in verified code paths.

- **Location:** `resume()` in exec (`interrupted.rs:268`) — `admission_time` oracle decouples from `clock::now()`
  - **Description:** The original `ReadyThread::from_state()` calls `clock::now()` internally to set the admission time. The verified `resume()` takes `admission_time` as a ghost oracle parameter with only a `>= 0` precondition. This means the verification cannot guarantee that the admission time corresponds to an actual clock reading. The `spec_admission_time_valid()` spec exists but is never required by any precondition or postcondition — it's advisory only. A caller could pass any non-negative value without violating the contract.
  - **Suggested Fix:** This is an inherent boundary issue (clock is HAL). However, document in the postcondition that the oracle pattern is used, and consider adding a `spec_admission_time_valid()` requirement in a cross-module integration proof, so the link to `clock::now()` is not purely comments.

### Medium

- **Location:** Boundary model of `InterruptedProcess` in `runnable.spec.rs:136-145` vs primary model in `interrupted.spec.rs`
  - **Description:** The `InterruptedProcessView` in `runnable.spec.rs` omits `sleeping_thread_ids`. This boundary inconsistency is documented in comments but means cross-module proofs involving interrupted processes with sleeping threads (created via `from_sleeping`) cannot use the runnable module's boundary model. If a cross-module proof links the `RunnableProcess` creation back to the `InterruptedProcess` source including sleeping threads, the models won't align without manual bridging.
  - **Suggested Fix:** Add `sleeping_thread_ids` to the `InterruptedProcessView` in `runnable.spec.rs` for model parity, or add a bridging lemma that maps between the two views.

- **Location:** `find_thread()` / `find_thread_mut()` in exec (`interrupted.rs:397-429`)
  - **Description:** These are purely ghost/spec-level functions that directly compute `spec_find_thread()`. They do not model the executable `iter().find()` search logic from the original. The original performs three sequential linear scans across `interrupted_threads`, `sleeping_threads`, and `zombie_threads`. The iterator-based search predicate (`thread.id() == tid`) and collection ordering are trusted, not verified. A bug in the original (e.g., searching zombie before sleeping) would not be caught. The `lemma_find_thread_refinement_assumption` (proof line 224) correctly documents this but it is a pure documentation lemma — it only restates the spec definition, it does not actually prove anything about the original code.
  - **Suggested Fix:** This is a known Verus limitation (reference-typed returns). The documentation is thorough. Accept as a trust boundary but tag it in a trust-boundary inventory for future re-verification if Verus capabilities expand.

- **Location:** `wf()` in spec (`interrupted.spec.rs:207-215`) — no upper bound on thread counts
  - **Description:** The well-formedness predicate requires `interrupted_thread_ids.len() >= 1` but places no upper bound on thread list lengths. The real kernel has finite memory and bounded thread counts. While this is unlikely to cause unsoundness (ghost sequences are unbounded by nature), it means the verification doesn't capture resource exhaustion scenarios.
  - **Suggested Fix:** Low priority. If resource bounds are important for the verification target, add `spec_max_threads` constant and bound total thread count in `wf()`. Otherwise, accept as out-of-scope for functional correctness.

### Low

- **Location:** `state_mut()` in exec (`interrupted.rs:231-243`) — frame condition is trivially satisfied
  - **Description:** The original `state_mut()` returns `&mut ProcessState`, allowing the caller to mutate any field of `ProcessState` (e.g., capabilities, memory map). The verification model abstracts `ProcessState` to just PID, so the frame condition (`self unchanged`) holds trivially. Any mutation of `ProcessState` fields other than PID is invisible to the verification. This is correctly documented in the trust boundary section.
  - **Suggested Fix:** If `ProcessState` fields beyond PID become verification-relevant, expand the model. Currently acceptable given the abstraction level.

- **Location:** `interrupt()` standalone function in exec (`interrupted.rs:450-456`)
  - **Description:** The standalone `interrupt()` function is modeled with correct ID preservation and reason tagging. The original calls `SleepingThread::interrupt(InterruptReason::Killed)`, which delegates to `InterruptedThread::from_state(self.state, reason)`. The model captures the essential contract. Minor nit: the return type `(Ghost<int>, Ghost<int>)` (tid, reason) is a simplification — the original returns a full `InterruptedThread` struct. This is fine for the abstraction level.
  - **Suggested Fix:** None needed.

- **Location:** `RunnableProcess::wf()` in spec (`interrupted.spec.rs:278-293`) — duplicated helper functions
  - **Description:** `spec_no_duplicates` and `spec_seqs_disjoint` are defined identically on both `InterruptedProcess` and `RunnableProcess`. This duplication could lead to divergence if one is updated and the other is not.
  - **Suggested Fix:** Consider extracting these into a shared spec module or using a trait. Low priority since both definitions are simple and identical.

## Positive Observations

- **Verification passes cleanly:** All 22 verification conditions pass with zero errors, no `assume`, `external_body`, or `trusted` annotations in executable or proof code.
- **Well-formedness invariant is strong:** The `wf()` predicate enforces both intra-list uniqueness (`spec_no_duplicates`) and inter-list disjointness (`spec_seqs_disjoint`), faithfully modeling Rust's ownership semantics for thread structures across multiple `NonEmptyVecDeque` collections.
- **`resume()` proof is thorough:** The `resume()` function includes detailed proof steps: tail no-duplicates preservation, front-not-in-tail, tail disjointness with sleeping/zombie, singleton ready no-duplicates, and ready-disjoint-from-all-other-lists. All six pairwise disjointness conditions on the resulting `RunnableProcess` are established.
- **PID immutability is proven:** Every operation's postcondition explicitly asserts `result.spec_pid() == self.spec_pid()` (or the pre-state's PID for mutating operations).
- **Trust boundary documentation is excellent:** The module header and spec file thoroughly document what is trusted vs. verified, including the `find_thread` refinement gap, the clock oracle pattern, the `ProcessState` abstraction, and the boundary model inconsistency. This is a model for trust documentation.
- **Split quality is good:** Spec, proof, and exec are cleanly separated. Spec contains only `open spec fn` definitions, view types, and constants. Proof contains only lemmas. Exec contains the implementations with inline proof blocks only where needed (in `resume()`).
- **Function coverage is complete:** All 8 items from the original (struct definition, `new`, `from_sleeping`, `state`, `state_mut`, `resume`, `find_thread`, `find_thread_mut`, and standalone `interrupt`) have corresponding verified models.

## Summary

The verification of `interrupted_process` is high quality. All functions are covered, the well-formedness invariant correctly models Rust ownership semantics via uniqueness and disjointness, and the `resume()` state transition proof is rigorous. The two high-priority items — unmodeled `interrupt_reason` mutation during `resume()` and the advisory-only clock oracle — represent real gaps but are reasonable given the abstraction level. The `find_thread` spec-only model is a known Verus limitation, well-documented. The cross-module boundary model inconsistency (missing `sleeping_thread_ids` in `runnable.spec.rs`) should be addressed if integration proofs are planned. Overall, this is a solid verification with clearly documented trust boundaries.
