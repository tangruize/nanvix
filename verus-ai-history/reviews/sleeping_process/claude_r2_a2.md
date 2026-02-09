# Review: sleeping_process (claude-opus-4.6)

## Grade: A

## Verification Result

**20 verified, 0 errors** (up from 18 in prior round). No `assume` statements. Two `external_body` annotations on `state()` and `state_mut()`, both justified (modeling reference-returning accessors with frame conditions).

## Previous Issue Disposition

### High #1: `add_thread()` missing uniqueness precondition — **FIXED** ✅
The prover added exactly the suggested preconditions (exec, lines 478–479):
```
!Self::spec_seq_contains(self.sleeping_thread_ids@, ready_tid@),
!Self::spec_seq_contains(self.zombie_thread_ids@, ready_tid@),
```
This correctly models Rust's ownership guarantee that a `ReadyThread` cannot simultaneously exist in the sleeping or zombie lists. Verified: the precondition is present and the function passes verification.

### High #2: Boundary `RunnableProcess::wf()` / `InterruptedProcess::wf()` too weak — **Downgraded to Low (Observation), Justified by Project Convention**
The boundary wf() predicates remain unchanged (spec, lines 236–238 and 248–250). However, investigation of sibling modules reveals this is a **project-wide convention**:
- `runnable.spec.rs`: `RunnableProcess::wf()` checks lengths/counts only (lines 346–357); uniqueness is in a separate `wf_strict()` (lines 362–376).
- `running.spec.rs`: `RunningProcess::wf()` checks counts only (lines 349–354); disjointness in `wf_strict()` (lines 362–376).
- All boundary types across modules use minimal wf() (`len() >= 1` or `true`).

The sleeping module's own `SleepingProcess::wf()` already includes no-dup/disjoint, which is stronger than sibling modules' own wf(). The postconditions on each function explicitly specify thread list contents, so consumers can derive uniqueness from postconditions without relying on boundary wf(). This is an intentional design split: `wf()` = structural baseline, `wf_strict()` = full ownership model.

**Verdict:** Not a defect; consistent project convention. No action needed.

### Medium #1: `wakeup()` missing explicit tid-absence postcondition — **FIXED** ✅
Added (exec, line 289):
```
&& !Self::spec_seq_contains(rp.sleeping_thread_ids@, tid@)
```
Supported by new proof lemma `lemma_remove_at_removes_element` (proof, lines 125–149) which proves that removing an element from a no-duplicates sequence means it no longer appears. The proof is sound: uses contradiction — if the value persists, map back to original index, derive two distinct indices with same value, contradicting no_duplicates.

### Medium #2: `wakeup_alarm()` oracle trust boundary — **Acknowledged, Accepted Trade-off**
No changes to alarm modeling. The alarm comparison logic (`now >= alarm`) remains in the trust boundary. The structural oracle constraints (length/content conservation, no-dup, disjoint, subsequence ordering) are unchanged and thorough. Given that the per-thread alarm check is a simple integer comparison (not complex logic), this remains a reasonable engineering trade-off. The module header and spec file document this trust boundary clearly (exec lines 46–49; spec lines 40–43).

### Medium #3: `find_thread_mut()` model fidelity — **FIXED** ✅
Added explicit trust documentation in spec file (lines 47–50):
```
// - `find_thread_mut()` yields `&mut` access; any mutation through it is
//   **outside** the scope of this verification model. Callers must ensure
//   that mutations preserve wf() as a caller-side proof obligation.
```
This correctly scopes the trust boundary and makes the caller's obligation explicit.

### Low #1: `state_mut()` external_body — **Unchanged, Acceptable**
Still uses `self.wf() == old(self).wf()` biconditional. This matches the original's unconditional `&mut` access pattern. No action needed.

### Low #2: `wakeup_alarm()` Err path comment — **FIXED** ✅
Added inline comment (exec, lines 449–450):
```
// No alarm expired: the process remains sleeping with all state unchanged.
// sleeping_count, sleeping_thread_ids, and zombie_thread_ids are identity-preserved.
```

### Low #3: Missing `lemma_remove_at_no_duplicates` — **FIXED** ✅
Added complete proof (proof, lines 103–121) using `assert forall` pattern with index mapping back to original sequence. Proof is sound: maps result indices `i,j` to original indices `si,sj` where `si < sj`, then invokes input `spec_no_duplicates` to derive `s[si] != s[sj]`, establishing `result[i] != result[j]`.

## New Issues Check

### Critical
None.

### High
None.

### Medium
None.

### Low

- **Location:** `wakeup_alarm()` oracle trust boundary (exec, lines 383–458)
  **Description:** Carried forward from previous review. The alarm partition correctness is delegated to oracle parameters with strong structural constraints but no semantic check that interrupted threads actually have expired alarms. This is the largest unverified gap in the module.
  **Status:** Accepted trust boundary, clearly documented. No action required unless alarm modeling is added project-wide.

- **Location:** `state_mut()` external_body (exec, lines 193–204)
  **Description:** Carried forward. The `self.wf() == old(self).wf()` biconditional is technically weaker than `requires self.wf(), ensures self.wf()`. No precondition prevents calling on a non-wf process.
  **Status:** Matches original code's unconditional accessor pattern. Acceptable.

## Positive Observations

- **All substantive issues from round 1 were genuinely fixed.** The prover addressed 5 of 8 issues with real code/proof changes, and the 3 unaddressed items are justified by project conventions or accepted as trade-offs.

- **New proof lemmas are sound and well-structured.** `lemma_remove_at_removes_element` (proof, lines 125–149) uses a clean proof-by-contradiction pattern. `lemma_remove_at_no_duplicates` (proof, lines 103–121) uses a correct index-mapping `assert forall` pattern. Both are non-trivial proofs that add genuine verification value.

- **Verification count increased from 18 to 20**, reflecting the two new proof lemmas. All obligations discharge cleanly.

- **Full function coverage maintained:** All 9 public functions from the original source have verified counterparts with meaningful specifications.

- **`SleepingProcess::wf()` is the strongest wf() in the project**, including no-dup and disjointness where sibling modules only have this in `wf_strict()`. This is a positive design choice for this module.

- **PID immutability, thread conservation, and state machine correctness** are all proven across every transition.

- **Clean spec/proof/exec separation** is maintained with no mixing of concerns.

- **The `add_thread()` precondition now correctly models Rust ownership**, closing the most significant semantic gap from round 1.

- **The `wakeup()` postcondition now explicitly states the key safety property** — the woken thread is no longer sleeping — making the API self-documenting for downstream consumers.

## Summary

The prover addressed all substantive issues from round 1. The two High issues are resolved: `add_thread()` now enforces thread ID uniqueness via preconditions, and the boundary wf() concern is justified as a project-wide convention (with uniqueness handled via explicit postconditions or `wf_strict()` in sibling modules). The Medium issues are either fixed (tid-absence postcondition, find_thread_mut documentation) or accepted as clearly-documented trust boundaries (alarm oracle). All new proof lemmas are sound.

The remaining items are two Low-priority observations: the alarm oracle trust boundary (inherent to the modeling approach) and the state_mut() biconditional pattern (matches the original code). Neither represents a verification defect.

The module now provides strong, well-documented verification of the SleepingProcess state machine with 20 verified obligations, no assumes, and minimal external_body usage restricted to reference-returning accessors.
