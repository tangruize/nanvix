# Review: sleeping_process (claude-opus-4.6)

## Grade: A-

## Verification Result

**18 verified, 0 errors** — all proof obligations discharged.

## Issues Found

### Critical

None.

### High

- **Location:** `add_thread()` (exec, line 466)
  **Description:** No precondition requires `ready_tid` to be distinct from existing sleeping and zombie thread IDs. In the original code, Rust's type system prevents a `ReadyThread` from already existing inside the sleeping/zombie lists (ownership semantics). In the verification model, thread IDs are plain `int` values with no ownership tracking, so nothing prevents a caller from passing a `ready_tid` that collides with an existing sleeping or zombie thread ID. The resulting `RunnableProcess` would have a duplicate thread ID across its ready and sleeping lists, violating the implicit invariant that all thread IDs in a process are unique.
  **Suggested Fix:** Add preconditions:
  ```
  requires
      self.wf(),
      !Self::spec_seq_contains(self.sleeping_thread_ids@, ready_tid@),
      !Self::spec_seq_contains(self.zombie_thread_ids@, ready_tid@),
  ```

- **Location:** `RunnableProcess::wf()` / `InterruptedProcess::wf()` (spec, lines 233–247)
  **Description:** Boundary type well-formedness predicates are too weak. `RunnableProcess::wf()` only checks `ready_thread_ids@.len() >= 1`; `InterruptedProcess::wf()` only checks `interrupted_thread_ids@.len() >= 1`. Neither enforces no-duplicate or disjointness invariants across their multiple thread lists (ready, interrupted, sleeping, zombie). This means the postconditions claiming `result.wf()` for `terminate()`, `wakeup()`, `wakeup_alarm()`, and `add_thread()` guarantee very little about the structural integrity of the output. A consumer of these boundary types cannot rely on thread ID uniqueness without re-proving it.
  **Suggested Fix:** Strengthen boundary wf() predicates to include no-duplicate and disjointness constraints across all thread lists, mirroring what the sibling modules' own wf() predicates would require. At minimum:
  ```
  pub open spec fn wf(&self) -> bool {
      &&& self.ready_thread_ids@.len() >= 1
      &&& SleepingProcess::spec_no_duplicates(self.ready_thread_ids@)
      &&& SleepingProcess::spec_no_duplicates(self.sleeping_thread_ids@)
      &&& SleepingProcess::spec_no_duplicates(self.zombie_thread_ids@)
      &&& SleepingProcess::spec_seqs_disjoint(self.ready_thread_ids@, self.sleeping_thread_ids@)
      &&& SleepingProcess::spec_seqs_disjoint(self.ready_thread_ids@, self.zombie_thread_ids@)
      &&& SleepingProcess::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
  }
  ```

### Medium

- **Location:** `wakeup()` postcondition (exec, lines 271–299)
  **Description:** The postcondition on the `Ok` path uses an existential (`exists|idx: int| ...`) to describe the removal but does not explicitly state that `tid@` is absent from the resulting sleeping list. While this follows logically from `spec_no_duplicates` on the original list and `spec_remove_at`, a consumer of this function must independently derive this fact. An explicit postcondition would make the API easier to use downstream and would directly capture the key safety property: the woken thread is no longer sleeping.
  **Suggested Fix:** Add an explicit postcondition clause:
  ```
  && !Self::spec_seq_contains(rp.sleeping_thread_ids@, tid@)
  ```

- **Location:** `wakeup_alarm()` oracle trust boundary (exec, lines 378–451)
  **Description:** The alarm-based partition (`now >= alarm` per thread) is entirely delegated to oracle parameters. While the structural constraints (length conservation, content conservation, no-duplicate, disjointness, subsequence ordering) are thorough and well-crafted, the actual correctness of the partition — that `interrupted_ids` corresponds to threads whose alarms have expired and `remaining_ids` to those whose alarms have not — cannot be verified. This is the largest gap in the verification; a bug in the alarm comparison logic would not be caught.
  **Suggested Fix:** Consider modeling `alarm: Option<int>` as a ghost field on each thread entry (e.g., `Ghost<Seq<Option<int>>>` parallel to `sleeping_thread_ids`). The partition precondition could then require:
  ```
  forall|i: int| 0 <= i < interrupted_ids@.len() ==>
      alarm_of(interrupted_ids@[i]).is_some() && now >= alarm_of(interrupted_ids@[i]).unwrap()
  ```
  This would close the gap without requiring exec-level alarm data. Alternatively, document this as an accepted trust boundary if the effort is not justified.

- **Location:** `find_thread_mut()` model fidelity (exec, lines 539–551)
  **Description:** The original `find_thread_mut()` returns `Option<ThreadRefMut<'_>>`, allowing the caller to mutate the found thread in-place (e.g., changing thread-level properties). The verified version's postcondition asserts a complete frame condition (`self` unchanged), which is correct within the ID-only model but does not account for any mutations the caller makes through the returned mutable reference. If a caller uses the mutable reference to change a thread's identity or membership, this would not be captured.
  **Suggested Fix:** Document this as an explicit trust assumption in the spec file. Consider adding a ghost "mutation token" pattern that requires the caller to prove the thread's ID is preserved after mutation, if Verus gains support for mutable borrows in the future.

### Low

- **Location:** `state_mut()` external_body (exec, lines 193–204)
  **Description:** The postcondition `self.wf() == old(self).wf()` is technically an equality of booleans rather than a preservation guarantee. If `old(self).wf()` is false, this allows `self.wf()` to also be false, which is correct but could be stronger. Since the function requires no precondition on wf(), a caller could invoke `state_mut()` on a non-wf process.
  **Suggested Fix:** Either add `requires self.wf()` and strengthen to `ensures self.wf()`, or leave as-is since this matches the original's unconditional `&mut` access. Current form is acceptable.

- **Location:** `wakeup_alarm()` Err path — `sleeping_count` (exec, line 448)
  **Description:** On the `Err` path, the returned `SleepingProcess` constructs `sleeping_count: self.sleeping_count`. The postcondition verifies `sp.sleeping_count == self.sleeping_count`. This is correct, but the proof relies on the `!has_expired ==> remaining_ids@ =~= self.sleeping_thread_ids@` precondition to ensure consistency with `wf()`. The coupling is implicit and could be made clearer with a comment.
  **Suggested Fix:** Add a brief inline comment or proof assertion noting why `sleeping_count` is still consistent with the sleeping list length on the Err path.

- **Location:** Proof file — missing `lemma_remove_at_no_duplicates` (proof)
  **Description:** There is no lemma proving that `spec_remove_at` on a no-duplicate sequence produces a no-duplicate sequence. While this property is not needed for current postconditions (since the result goes into `RunnableProcess` whose `wf()` doesn't check duplicates), it would be valuable if boundary `wf()` predicates are strengthened.
  **Suggested Fix:** Add:
  ```
  pub proof fn lemma_remove_at_no_duplicates(s: Seq<int>, idx: int)
      requires
          0 <= idx < s.len(),
          Self::spec_no_duplicates(s),
      ensures
          Self::spec_no_duplicates(Self::spec_remove_at(s, idx)),
  ```

## Positive Observations

- **Full function coverage:** All 9 public functions from the original source have verified counterparts with meaningful specifications. No functions were skipped or stubbed without justification.

- **Well-formedness invariant is strong and complete:** The `wf()` predicate captures the essential structural invariants: non-empty sleeping list, count consistency, no-duplicate within lists, and cross-list disjointness. This directly models Rust's ownership semantics for thread collections.

- **Oracle design for `wakeup_alarm()` is thorough:** The oracle preconditions include length conservation, content conservation, partition integrity (no-dup + disjoint), stable ordering (subsequence), and identity preservation on the no-expiry path. This is a well-engineered trust boundary that constrains the oracle tightly.

- **Clean spec/proof/exec split:** Specifications (View types, spec functions, wf()) are cleanly separated from proof lemmas and exec code. The `include!()` pattern keeps the files focused.

- **`spec_is_subsequence` models stable partition ordering:** This captures an important property of the original `wakeup_alarm()` — that thread ordering is preserved through partitioning. Many verifications would skip this.

- **PID immutability is proven across all operations:** Every state transition ensures the process identifier is preserved, which is a key correctness property for an OS process state machine.

- **`terminate()` specification is tight:** The postcondition precisely states that `interrupted_thread_ids == sleeping_thread_ids` (identity, not just length), ensuring no threads are lost or fabricated.

- **Frame conditions on `state_mut()` and `find_thread_mut()` are correct:** Both external/ghost functions properly preserve all ghost state, modeling the fact that these accessors don't modify thread list membership.

## Summary

The verification of `sleeping_process` is thorough and well-structured, covering all 9 public functions with meaningful specifications. The core state machine transitions (`terminate`, `wakeup`, `wakeup_alarm`, `add_thread`) are verified with strong postconditions that capture PID preservation, thread conservation, and list structure. The verification passes cleanly (18/0).

The two most significant gaps are: (1) boundary type `wf()` predicates are minimal, meaning cross-list uniqueness is not guaranteed in output types, and (2) `add_thread()` lacks a precondition preventing thread ID collisions — both stem from the same root issue of not modeling ownership semantics for thread IDs at the boundary. Strengthening boundary `wf()` predicates and adding the missing uniqueness precondition to `add_thread()` would elevate this to an A/A+ grade. The oracle-based alarm partition in `wakeup_alarm()` is well-constrained structurally but leaves alarm-comparison correctness in the trust boundary, which is an acceptable engineering trade-off given the simplicity of the underlying comparison.
