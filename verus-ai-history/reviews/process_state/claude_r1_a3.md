# Review: process_state (claude-opus-4.6) — Round 3

## Grade: A

## Previous Issues Disposition

### From R2 — Medium: ref count initial value off by 1

**FIXED.** Verified by inspection:

- `get_mutex` new entry: `self.ghost_mutexes@.insert(mutex_addr@, 2nat)` (exec line 314). Postcondition: `!already_present ==> self.spec_mutex_ref_count(mutex_addr@) == 2` (line 276). This correctly models `Mutex::new()` + `.clone()`: the BTreeMap holds one `Arc` reference (count 1), and the returned clone adds another (count 2). ✓
- `get_cond` new entry: `self.ghost_conditions@.insert(cond_addr@, 2nat)` (exec line 461). Same reasoning: `Condvar::new()` + `.clone()` = 2. ✓
- Trust assumption T3 updated: "New entries start at 2 (one Arc in the BTreeMap, one returned clone)" (exec line 62). ✓
- Proof lemmas renamed: `lemma_new_mutex_ref_count_is_two` / `lemma_new_cond_ref_count_is_two` (proof lines 362-386). ✓

The initial value of 2 now directly models `Arc::strong_count()` after the `entry().or_insert_with().clone()` chain in the original. The threshold constants (`MUTEX_REMOVE_THRESHOLD = 2`, `COND_REMOVE_THRESHOLD = 1`) remain unchanged and now correctly correspond to the real `Arc::strong_count()` at creation time.

### From R2 — Low: stub signature mismatch

**ADDRESSED.** Documentation block added at exec lines 635-641: "These stubs have simplified signatures compared to the originals. They are not intended to be API-compatible replacements. Their sole purpose is to assert frame conditions (non-interference with verified ghost state)." Adequate documentation of the limitation.

### From R2 — Low: get_mutex capacity check on existing keys

**N/A.** Correctly identified in R2 as faithful modeling of original behavior (the original also rejects existing-key lookups when at capacity). No fix needed or attempted.

## New Issues Found

### Low

- **Location:** Trust assumption T3 documentation (exec: lines 62-66)
  **Description:** T3 states "dropping the caller's clone decrements it" but the verified API provides no decrement operation. The model only increments ref counts (via `get_mutex`/`get_cond`) and never decrements them (except via entry removal in `put_mutex`/`put_cond`). This means `model_ref_count` is an upper bound on `real_arc_strong_count` — it tracks increments but not implicit `Drop`s. The T3 documentation overclaims by suggesting the model tracks decrements. In practice, this is a safe over-approximation: since `model_ref ≥ real_ref`, the model is **more conservative about removal** than reality (it may decline to remove when the real code would). This doesn't introduce unsoundness — it means some real-system behaviors (those involving clone drops between `get` and `put`) cannot be represented in the model.
  **Suggested Fix:** Revise T3 to say: "The ghost ref count models the cumulative clone count (initial + subsequent `get_*` calls). Since implicit `Arc::Drop` is not modeled, the ghost count is an upper bound on `Arc::strong_count()`. The removal threshold check is therefore conservative: the model may not represent removals that the real code performs after callers drop their clones, but it never permits removals that the real code would not." Alternatively, add an explicit `drop_mutex_ref(addr: Ghost<int>)` method with the postcondition `self.spec_mutex_ref_count(addr) == old(self).spec_mutex_ref_count(addr) - 1` and appropriate `wf` guard (`ref_count > 1`, since the BTreeMap always holds one reference).

## Positive Observations

- **Verification passes cleanly:** 43 verified, 0 errors. Stable across all three rounds.
- **No `assume` or `trusted` in core logic.** All `external_body` annotations are strictly confined to frame-condition stubs for opaque boundary types (Vmem, EventOwnership, Mailbox, IoMemoryRegion, AnyIoPort). Trust assumption T4 explicitly documents this.
- **Reference count model is now faithful to the original.** The initial value of 2 directly corresponds to `Arc::strong_count()` after `Mutex::new().clone()`. The threshold constants (2 for mutexes, 1 for condvars) match the original `extract_if` predicates exactly. The model is sound — it may be incomplete (missing implicit drops) but never introduces false proofs.
- **Strong `wf()` invariant.** Includes capacity bounds (`mutex_count <= MUTEX_MAX`, `cond_count <= COND_MAX`), ghost map domain consistency (`dom().len() == count`), and positive ref-count invariants for all entries. This rules out a wide class of invalid states.
- **Comprehensive frame conditions.** All mutating functions specify exact frame conditions for every verified field. The `forall|a: int| a != addr ==> spec_has_*(a) == old(self).spec_has_*(a)` pattern correctly proves non-interference with other map entries.
- **Clean spec/proof/exec separation.** Specs define abstract state accessors and invariants. Proofs provide reusable lemmas. Exec code contains implementation with contracts. No verification logic leaks across boundaries.
- **Well-documented trust assumptions.** T1-T4 are specific, verifiable claims. T1 references the exact config file. T2 states the BTreeMap model. T3 describes the Arc model. T4 covers external_body stubs.
- **All R1 critical/high issues fully resolved.** Reference counting, capacity constants, PMIO removal semantics, and wf() strengthening are all substantively addressed and verified.

## Summary

All issues from R1 (7 issues) and R2 (3 issues) have been addressed. The verification is sound, well-structured, and well-documented. The one remaining Low-priority issue is a documentation overclaim in T3 about modeling implicit Arc drops — the model is actually a safe upper-bound approximation, which is conservative but sound.

The verification successfully proves:
- PID immutability across all operations
- Capability delegation correctness with wf preservation
- Bounded mutex/condvar management with reference-counted conditional removal
- Correct error codes for all failure paths
- I/O port sequence consistency with single-element removal
- Frame conditions for all operations including opaque boundary types
- Capacity invariants as part of well-formedness

This is a solid verification of the ProcessState management protocol suitable for an OS kernel component.
