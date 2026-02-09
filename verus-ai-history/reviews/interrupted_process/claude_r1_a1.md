# Review: interrupted_process (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **Boundary RunnableProcess lacks disjointness in wf()** (spec — `interrupted.spec.rs:213-216`)
  - **Description:** The boundary `RunnableProcess::wf()` only checks `ready_thread_ids@.len() >= 1`. It does not require thread-ID uniqueness or pairwise disjointness across ready/interrupted/sleeping/zombie lists. This means `resume()` proves the result is "well-formed" under a very weak definition. The disjointness proof lemmas (`lemma_tail_disjoint_sleeping`, `lemma_tail_disjoint_zombie`, `lemma_front_not_in_tail`) exist in the proof file but are never invoked — they float as standalone lemmas rather than strengthening the postcondition.
  - **Suggested Fix:** Either (a) strengthen the boundary `RunnableProcess::wf()` to include no-duplicates and pairwise-disjoint conditions mirroring `InterruptedProcess::wf()`, or (b) add explicit postconditions on `resume()` asserting the disjointness properties (`spec_no_duplicates` / `spec_seqs_disjoint` on each output list pair). The proof infrastructure already exists in the proof file — it just needs to be wired in.

### Medium

- **Cross-module boundary model inconsistency for `InterruptedProcess`** (spec — `runnable.spec.rs:138-145` vs `interrupted.spec.rs:56-65`)
  - **Description:** The `InterruptedProcess` boundary model inside `runnable.spec.rs` omits `sleeping_thread_ids`, while the primary model in `interrupted.spec.rs` includes it. An `InterruptedProcess` created via `from_sleeping()` can hold sleeping threads, but the runnable module's boundary has no field for them. If cross-module linking is attempted, properties involving sleeping threads in an `InterruptedProcess` cannot be expressed through the runnable boundary.
  - **Suggested Fix:** Add `sleeping_thread_ids: Seq<int>` to the `InterruptedProcess` boundary model and `InterruptedProcessView` in `runnable.spec.rs`, or document explicitly that the runnable boundary only models the `new()` path (no sleeping threads).

- **`find_thread`/`find_thread_mut` are spec-only — exec search logic is unverified** (exec — `interrupted.rs:280-311`)
  - **Description:** Both functions return `Ghost<Option<int>>` and compute results purely in spec mode (`Ghost(self.spec_find_thread(tid@))`). The original code performs linear searches through `iter().find(...)` across three collections with a specific priority order (interrupted → sleeping → zombie). The Verus model captures the search order in `spec_find_thread` but doesn't verify any executable search implementation. This means any bug in the actual iterator-based search logic (e.g., wrong predicate, wrong collection order) would not be caught.
  - **Suggested Fix:** This is a known Verus limitation (reference-typed return values). Document this gap explicitly in the trust boundary section. If Verus ever supports executable iteration over ghost sequences, revisit. For now, the spec-only modeling is the pragmatic choice.

- **Boundary RunnableProcess omits `ready_admission_times`** (exec — `interrupted.rs:77-88` vs `runnable.rs`)
  - **Description:** The real `RunnableProcess` has `ready_admission_times: Ghost<Seq<int>>` as a parallel array to `ready_thread_ids`, and its `wf()` requires matching lengths and non-negative values. The boundary model in `interrupted.rs` omits this field entirely. This means `resume()` doesn't specify the initial admission time for the newly-readied thread. While this is a property of the thread (not the process state), it creates an unconstrained ghost field when the boundary result is consumed by downstream proofs.
  - **Suggested Fix:** Add `ready_admission_times: Ghost<Seq<int>>` to the boundary `RunnableProcess` and constrain it in the `resume()` postcondition (e.g., `result.ready_admission_times@.len() == 1 && result.ready_admission_times@[0] >= 0`).

### Low

- **`state_mut()` external_body postcondition doesn't explicitly ensure wf() preservation** (exec — `interrupted.rs:200-210`)
  - **Description:** The `state_mut()` frame condition preserves all four ghost fields, so `wf()` is implicitly preserved. However, the postcondition doesn't state `self.wf()` explicitly. A caller would need to reason through the field equalities to conclude well-formedness is maintained. Adding `self.wf()` to ensures (given `old(self).wf()` as a precondition) would make the contract self-documenting.
  - **Suggested Fix:** Add `requires old(self).wf()` and `ensures self.wf()` to the `state_mut()` contract, or invoke `lemma_mutation_frame_preserves_wf` in the proof.

- **Proof lemmas for construction are redundant** (proof — `interrupted.proof.rs:30-79`)
  - **Description:** `lemma_new_is_wf` and `lemma_from_sleeping_is_wf` have empty bodies — the SMT solver discharges them automatically. The constructors themselves already have `ensures result.wf()` that Verus verifies. These lemmas restate what the ensures clauses already guarantee and are never invoked elsewhere.
  - **Suggested Fix:** Keep them if they serve as regression guards or documentation, but note they're redundant with the constructor postconditions.

## Positive Observations

- **100% function coverage.** All 7 original functions (`new`, `from_sleeping`, `state`, `state_mut`, `resume`, `find_thread`, `find_thread_mut`) plus the standalone `interrupt()` function are modeled and verified.
- **Zero assume statements.** No unjustified assumptions anywhere in exec, spec, or proof files. The only `external_body` annotations are on `state()`/`state_mut()` which return reference types inexpressible in Verus — fully justified and documented.
- **Strong wf() predicate.** The well-formedness invariant includes non-empty interrupted threads, per-list no-duplicates, and pairwise disjointness — correctly modeling Rust's ownership guarantee at the specification level.
- **Clean split quality.** Exec code has no proof artifacts; specs are purely declarative; proof lemmas are self-contained. The `include!` mechanism keeps the three files composable.
- **Thorough documentation.** Trust boundaries, modeling decisions, and verification limitations are all explicitly documented in file-level comments.
- **resume() postcondition is comprehensive.** It fully specifies PID preservation, the exact ready thread (front of interrupted list), the exact remaining interrupted list (tail/subrange), and preservation of sleeping/zombie threads. This is a strong, machine-checked specification of the state transition.
- **Semantic equivalence is correct.** The `NonEmptyVecDeque::pop_front` ↔ `subrange(1, len)` modeling faithfully captures the original's front-removal semantics, including the edge case where only one interrupted thread exists (resulting in an empty remaining list).
- **18/18 verification conditions pass** with no errors.

## Summary

This is a well-executed verification of the `InterruptedProcess` module. The modeling choices — abstracting threads as ghost integer IDs, using sequences for collections, and treating `ProcessState` as an opaque PID — are sound and well-documented. All original functions are covered with meaningful specifications.

The primary gap is that the boundary `RunnableProcess::wf()` is weak (no uniqueness/disjointness), so the proved postcondition of `resume()` doesn't guarantee the full structural integrity of the resulting process. The proof infrastructure to close this gap already exists in the proof file but isn't wired into the postcondition. Strengthening the boundary model or the `resume()` postcondition would elevate this from A- to A.

The cross-module boundary inconsistency (missing `sleeping_thread_ids` in the runnable module's `InterruptedProcess` boundary) should be resolved before attempting end-to-end verification across modules. The spec-only modeling of `find_thread`/`find_thread_mut` is a pragmatic Verus limitation, properly documented.

**Recommendations (priority order):**
1. Strengthen `resume()` postcondition with explicit disjointness/uniqueness (using existing proof lemmas).
2. Align the `InterruptedProcess` boundary model in `runnable.spec.rs` with the primary model.
3. Add `ready_admission_times` to the boundary `RunnableProcess`.
4. Add `self.wf()` to `state_mut()` ensures clause.
