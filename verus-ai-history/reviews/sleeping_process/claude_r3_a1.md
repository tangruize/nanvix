# Review: sleeping_process (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- None.

### Medium

1. **find_thread_mut frame condition prevents modeling mutation**
   - **Location:** `find_thread_mut()` in exec (`sleeping.rs:550-562`)
   - **Description:** The postcondition guarantees all fields are preserved (`self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@`, etc.), but the original returns `Option<ThreadRefMut<'_>>` which allows the caller to mutate the found thread in place. The verified model makes it impossible for callers to reason about mutations through the returned mutable reference — any caller that actually modifies a thread via `find_thread_mut` has no verification support. The spec file documents this as a trust boundary, but it means that any code path that mutates a sleeping/zombie thread through this accessor is entirely unverified.
   - **Suggested Fix:** Consider adding an alternative exec function (e.g., `mutate_sleeping_thread(tid, new_tid)`) that models the mutation effect on the thread list, with a postcondition that the old tid is replaced by new_tid and wf() is preserved. This would close the gap for common mutation patterns.

2. **Boundary type `RunnableProcess::wf()` is too weak to establish cross-list invariants**
   - **Location:** `RunnableProcess::wf()` in spec (`sleeping.spec.rs:236-238`)
   - **Description:** `RunnableProcess::wf()` only checks `ready_thread_ids@.len() >= 1`. It does not enforce uniqueness within or disjointness across its four thread lists (ready, interrupted, sleeping, zombie). After `wakeup()` produces a `RunnableProcess`, the postcondition individually asserts correct list contents, but the `rp.wf()` postcondition clause does not carry the combined cross-list disjointness property. A downstream consumer relying solely on `rp.wf()` would lack these guarantees.
   - **Suggested Fix:** Strengthen `RunnableProcess::wf()` to include `spec_no_duplicates` and `spec_seqs_disjoint` across all four lists, mirroring how `SleepingProcess::wf()` enforces these for its lists. Alternatively, add explicit disjointness postconditions to `wakeup()` and `add_thread()`.

3. **wakeup_alarm oracle fully abstracts alarm comparison logic**
   - **Location:** `wakeup_alarm()` in exec (`sleeping.rs:383-458`)
   - **Description:** The `has_expired`, `interrupted_ids`, and `remaining_ids` oracle parameters completely abstract away the alarm checking logic (`now >= alarm`). While the preconditions ensure the oracle constitutes a valid partition (length conservation, content inclusion, no duplicates, disjointness, subsequence ordering), there is no connection between the oracle and actual time values. A buggy caller could supply an incorrect partition (e.g., marking a non-expired alarm as expired) and the verification would still pass. The original's correctness depends on `now >= alarm` being evaluated correctly per-thread.
   - **Suggested Fix:** Document this as an explicit assumption in the trust boundary section. For higher assurance, model alarm values as a ghost field (e.g., `alarm_times: Ghost<Seq<Option<int>>>`) and add a precondition tying the partition to `now >= alarm_times[i]`, which would verify the partition logic against actual alarm data.

### Low

1. **`InterruptedProcess::wf()` lacks disjointness between interrupted and sleeping lists**
   - **Location:** `InterruptedProcess::wf()` in spec (`sleeping.spec.rs:248-249`)
   - **Description:** Like `RunnableProcess::wf()`, this boundary type only checks `interrupted_thread_ids@.len() >= 1`. After `terminate()` or `wakeup_alarm()`, the postconditions individually assert correct contents, but `ip.wf()` alone doesn't carry cross-list invariants.
   - **Suggested Fix:** Strengthen or document as a known gap to be enforced by the interrupted module's own verification.

2. **`wakeup()` postcondition uses existential for removed index**
   - **Location:** `wakeup()` postcondition in exec (`sleeping.rs:283-286`)
   - **Description:** The postcondition asserts `exists|idx: int| ... && rp.sleeping_thread_ids@ == Self::spec_remove_at(self.sleeping_thread_ids@, idx)`. Under `wf()` uniqueness, this existential has a unique witness, so the sleeping list is fully determined. However, the existential form may be harder for downstream proofs to work with compared to a direct equality with a specific spec function.
   - **Suggested Fix:** Consider adding a spec function `spec_remove_element(s, val)` that computes the unique index internally and returns the result, allowing the postcondition to use direct equality: `rp.sleeping_thread_ids@ == Self::spec_remove_element(self.sleeping_thread_ids@, tid@)`.

3. **View types defined but not used in postconditions**
   - **Location:** `SleepingProcessView`, `RunnableProcessView`, `InterruptedProcessView` in spec (`sleeping.spec.rs:63-98`)
   - **Description:** The spec file defines `View` implementations for all three types, but no postcondition or lemma uses the view types (e.g., `self@ == ...`). The view types add specification surface area without being exercised, which could become stale if the underlying specs change.
   - **Suggested Fix:** Either use view types in at least one postcondition or lemma (e.g., a `terminate()` postcondition expressed in terms of `SleepingProcessView` → `InterruptedProcessView` mapping), or remove them if they serve no purpose.

4. **`state_mut()` external_body postcondition `self.wf() == old(self).wf()` is a logical identity, not a frame condition**
   - **Location:** `state_mut()` in exec (`sleeping.rs:201`)
   - **Description:** The postcondition `self.wf() == old(self).wf()` says the boolean value of wf() is preserved, but since all constituent fields are individually asserted equal to their old values, this clause is redundant (it follows automatically). It doesn't add verification strength. A more useful frame condition would assert that no other ghost state was affected.
   - **Suggested Fix:** Remove the redundant `self.wf() == old(self).wf()` clause or replace it with an explicit `self.wf()` (since `old(self).wf()` is not required in the precondition anyway — `state_mut()` has no `requires` clause).

## Positive Observations

- **Complete function coverage:** All 9 public functions from the original source have verified counterparts, including the complex `wakeup_alarm()` with its two-path control flow.
- **Strong well-formedness invariant:** `wf()` includes non-empty sleeping list, no duplicates within lists, and disjointness across lists — faithfully modeling Rust's ownership semantics for thread collections.
- **Sound oracle design for `wakeup_alarm()`:** The oracle preconditions (length conservation + content inclusion + no duplicates + disjointness + subsequence ordering) tightly constrain the oracle to be a valid stable partition. This is a well-executed pattern for abstracting data-dependent branching.
- **Thorough proof lemmas:** The proof file includes 9 well-structured lemmas covering remove-at operations, construction well-formedness, find-thread consistency, and view equality. The `lemma_remove_at_removes_element` proof is particularly well-crafted with a proof-by-contradiction structure.
- **No `assume` statements or unjustified `external_body`:** The only two `external_body` functions (`state()`, `state_mut()`) return ghost values for `Box<ProcessState>` access, which is a justified trust boundary since ProcessState internals are outside this module's scope.
- **Clean spec/proof/exec separation:** Spec file contains all spec functions and views; proof file contains all lemmas; exec file contains implementations with minimal inline proof. Well-organized with section headers matching the project's coding standards.
- **Stable partition ordering verified:** The `spec_is_subsequence` predicate in `wakeup_alarm()` postconditions captures the important property that both partitions preserve the relative order from the original sleeping list, matching the original implementation's front-to-back processing.
- **20 verified, 0 errors:** All verification obligations discharge cleanly in ~5 seconds.

## Summary

This is a high-quality verification of the SleepingProcess module. All 9 public functions are covered with appropriate specifications, the well-formedness invariant faithfully models Rust ownership semantics through uniqueness and disjointness predicates, and the oracle-based approach for `wakeup_alarm()` is tightly constrained. The verification is sound with no `assume` statements and only two justified `external_body` functions.

The main gaps are in boundary type modeling (weak `wf()` for `RunnableProcess` and `InterruptedProcess`) and the inability to model mutations through `find_thread_mut()`. These are documented trust boundaries rather than bugs. The alarm comparison abstraction in `wakeup_alarm()` is a reasonable design choice given that the complex part (partition logic and thread conservation) is fully verified.

**Recommendations:**
1. Strengthen boundary type `wf()` predicates to include cross-list invariants, or coordinate with sibling module verifications to ensure these are enforced somewhere.
2. Consider adding a mutation-modeling function to close the `find_thread_mut()` gap for common mutation patterns.
3. For higher assurance on `wakeup_alarm()`, model alarm times as ghost data to tie the partition oracle to actual time values.
