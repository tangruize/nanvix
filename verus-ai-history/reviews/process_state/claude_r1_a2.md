# Review: process_state (claude-opus-4.6) — Round 2

## Grade: A-

## Previous Issues Disposition

### Critical Issues (from R1) — Both FIXED

1. **`put_mutex` unconditional removal** — **FIXED.** Now takes `ref_count_at_threshold: bool` oracle parameter tied via precondition to the ghost ref count: `contains ==> (ref_count_at_threshold == (old(self).spec_mutex_ref_count(mutex_addr@) <= Self::MUTEX_REMOVE_THRESHOLD()))`. Removal is conditional on the threshold (lines 382-386). Postconditions correctly distinguish the two cases: removal when at threshold (`!self.spec_has_mutex(mutex_addr@)`, count decremented) vs. retention when above threshold (entry remains, count unchanged). The `MUTEX_REMOVE_THRESHOLD` spec constant is correctly set to 2, matching the original's `reference_count() <= 2`. Verified.

2. **`put_cond` unconditional removal** — **FIXED.** Same pattern as `put_mutex`, with `COND_REMOVE_THRESHOLD = 1` matching the original's `reference_count() <= 1`. Verified.

### High Issues (from R1) — Both FIXED

3. **Capacity constants 256 vs 32** — **FIXED.** `MUTEX_MAX()` and `COND_MAX()` now return `32usize` (spec lines 151-158). `MUTEX_MAX_EXEC()` and `COND_MAX_EXEC()` also return `32usize` (exec lines 816-828). Matches `build/kernel_config.toml:40,45`. Doc comment references the config file. Verified.

4. **`remove_pmio` filter-all vs remove-first** — **FIXED.** Now takes `found_idx: Ghost<int>` and uses `subrange(0, found_idx@).add(subrange(found_idx@ + 1, len))` (exec line 612-614), correctly removing only the element at the found index. Postcondition now specifies `self.spec_pmio_count() == old(self).spec_pmio_count() - 1` (line 586). Precondition validates the ghost index: `found ==> 0 <= found_idx@ < old(self).ghost_pmio@.len() && old(self).ghost_pmio@[found_idx@] == port_number@`. Verified.

### Medium Issues (from R1) — All FIXED

5. **`wf()` missing capacity bounds** — **FIXED.** `wf()` now includes `self.mutex_count as nat <= Self::MUTEX_MAX() as nat` and `self.cond_count as nat <= Self::COND_MAX() as nat` (spec lines 130-131). Dedicated lemmas `lemma_mutex_count_bounded` and `lemma_cond_count_bounded` prove these bounds follow from `wf()`. Additionally, `wf()` now includes positive ref-count invariants: `forall|addr| self.ghost_mutexes@.contains_key(addr) ==> self.ghost_mutexes@[addr] > 0` (and similarly for condvars). Verified.

6. **12 functions omitted** — **FIXED.** Frame-condition stubs added for all 12 previously omitted functions as `external_body` methods (exec lines 629-809): `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`. Mutable stubs specify full frame conditions (preserving PID, capabilities, mutex/cond counts and maps, PMIO ports, and wf). Read-only stubs ensure `true`. Trust assumption T4 documents this. Verified (see new issue below regarding stub signatures).

7. **`get_mutex`/`get_cond` return type** — **FIXED.** Now return `Result<Ghost<nat>, Error>` where the ghost value is the new reference count. Postcondition specifies `result->Ok_0@ == self.spec_mutex_ref_count(mutex_addr@)` (exec line 273). This captures the correspondence between the returned value and the internal state. Verified.

8. **`ProcessRefMut`/`ProcessRef` not modeled** — **ADDRESSED.** Explicitly documented as out of scope in the module doc (exec lines 74-76, 84-85): "accessor wrappers for different process lifecycle states and are out of scope for this module's protocol verification." Reasonable scoping decision for enum dispatch wrappers that don't contain business logic.

### Low Issues (from R1) — All FIXED

9. **Conditional wf in set/clear_capability** — **FIXED.** Postconditions now simply assert `self.wf()` unconditionally (exec lines 196, 217). No longer uses the unnecessarily weak `old(self).capabilities.wf() ==> self.wf()`. Verified.

10. **Oracle pattern documentation** — **FIXED.** All oracle parameters have doc comments explaining their purpose (e.g., "Runtime result of BTreeMap::contains_key, tied to ghost state"). Adequate.

11. **Debug trait impl** — **N/A.** Correctly remains omitted.

## New Issues Found

### Medium

- **Location:** `get_mutex()` / `get_cond()` reference count initial value (exec: `process_state.rs:307-310`, `process_state.rs:453-456`)
  **Description:** When a new mutex is inserted, the model sets `ref_count = 1` (the BTreeMap entry). However, in the original code, `get_mutex` returns `.clone()` of the entry, which increments the real `Arc::strong_count()` to 2 immediately. Each subsequent `get_mutex` on an existing key both clones (incrementing Arc count) and increments the model's ref count by 1. This means `model_ref_count = real_arc_count - 1` at all times (in the absence of caller drops, which are not modeled).

  This creates a semantic gap at the `put_mutex` threshold check: the model removes when `model_ref <= 2`, but the real code removes when `real_ref <= 2`. Since `model_ref = real_ref - 1`, the model's condition is equivalent to `real_ref <= 3`, meaning the model is **more permissive about removal** than the original. Concretely: after creating a mutex and then calling `get_mutex` once more on the same address (model_ref=2, real_ref=3), the model says "remove" (2≤2) but the real code says "keep" (3>2).

  This doesn't affect the model's internal consistency (all proofs verify), but it means the ghost ref count doesn't faithfully correspond to `Arc::strong_count()`, which undermines trust assumption T3 ("The ghost `nat` value faithfully models `Arc::strong_count()`"). The oracle `ref_count_at_threshold` cannot be correctly populated from the real Arc count when the model is off by 1.

  **Suggested Fix:** Initialize new entries with `ref_count = 2` (accounting for the clone returned to the caller), so that `model_ref_count == real_arc_count` after `get_mutex`. Alternatively, document this offset explicitly and adjust `MUTEX_REMOVE_THRESHOLD` to 1 (so that model's `<= 1` with the -1 offset corresponds to reality's `<= 2`). For condvars, the initial ref count should also be 2 after clone, making `COND_REMOVE_THRESHOLD` 0 to correspond to reality's `<= 1`.

### Low

- **Location:** Frame-condition stubs (exec: `process_state.rs:629-809`)
  **Description:** The `external_body` stubs have simplified signatures that don't match the original function signatures. For example, `add_event_stub(&mut self)` omits the `ownership: EventOwnership` parameter, `receive_message_stub(&mut self)` omits `tid: ThreadIdentifier` and the `Option<Message>` return type, etc. While these stubs correctly document frame conditions, their signature mismatch means they cannot be directly composed with callers of the real API for end-to-end verification. They serve as proof obligations rather than usable specs.
  **Suggested Fix:** This is acceptable for the current scope. Document that these are frame-condition assertions, not API-compatible stubs. No code change strictly needed, but the limitation should be noted for anyone attempting to compose these with caller-side verification.

- **Location:** `get_mutex()` capacity check behavior (exec: `process_state.rs:293-296`)
  **Description:** Both the original and verified code return `OutOfMemory` when the map is at capacity, even if the requested address is already present (capacity check precedes the presence check). This means an existing mutex cannot be retrieved when the map is full. The verified code faithfully reproduces this behavior, which may be an overly conservative check in the original. This is not a verification defect — it's a faithful modeling of potentially suboptimal original logic.
  **Suggested Fix:** No action needed in verification. Could be flagged separately as a potential improvement to the original code.

## Positive Observations

- **Verification passes cleanly:** 43 verified, 0 errors (up from 33 in R1). Significant increase in coverage.
- **No `assume` or `trusted` in core logic:** All `external_body` annotations are confined to frame-condition stubs for opaque boundary types, explicitly documented under trust assumption T4.
- **Reference-count model is well-structured:** Ghost `Map<int, nat>` with ref-count semantics, threshold constants, and conditional removal logic are a substantial improvement. The `wf()` invariant now enforces positive ref counts for all entries.
- **Clean spec/proof/exec split:** Maintained from R1. The proof file now includes additional lemmas for ref-count preservation (`lemma_increment_mutex_ref_count_preserves_wf`, `lemma_put_mutex_at_threshold_removes`, etc.).
- **Trust assumptions clearly documented:** T1-T4 are well-specified. T3 (Arc ref counting) is the only one with a gap (the off-by-1 issue noted above).
- **Frame conditions for other-address preservation:** `get_mutex`/`put_mutex`/`get_cond`/`put_cond` now specify `forall|a: int| a != mutex_addr@ ==> spec_has_mutex(a) == old(self).spec_has_mutex(a)`, proving non-interference with other map entries.
- **Capacity bound lemmas:** Dedicated lemmas `lemma_mutex_count_bounded` and `lemma_cond_count_bounded` make the capacity invariant explicitly provable from `wf()`.

## Summary

All 7 issues from Round 1 (2 Critical, 2 High, 3 Medium, 2 Low) have been substantively addressed. The critical reference-counting and capacity constant issues are fixed. The `remove_pmio` semantic gap is resolved with index-based removal. Frame-condition stubs cover the previously omitted functions. The `wf()` predicate is strengthened with capacity bounds and positive-ref-count invariants.

One new Medium issue remains: the ghost reference count model initializes new entries at 1, but the real `Arc::strong_count()` is 2 after the `clone()` return in `get_mutex`. This off-by-1 means the model is slightly more permissive about removal than the original code, undermining the stated trust assumption T3. This is a correctness concern at the model-to-reality correspondence boundary, though the model's internal proofs are sound.

The verification is solid and well-documented. The module demonstrates good Verus practice with a clear verification scope, explicit trust assumptions, and comprehensive frame conditions. After fixing the ref-count initialization offset, this would be a strong A/A+ verification.

**Remaining recommendations:**
1. Fix ref-count initial value to 2 (or adjust thresholds) to match real `Arc::strong_count()` after clone.
2. Note stub signature limitations for future caller-side verification.
