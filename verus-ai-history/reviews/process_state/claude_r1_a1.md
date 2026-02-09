# Review: process_state (claude-opus-4.6)

## Grade: B-

## Issues Found

### Critical

- **Location:** `put_mutex()` (exec: `process_state.rs:306-316`)
  **Description:** The verified `put_mutex` unconditionally removes the mutex from the ghost map and decrements the count. The original (`mod.rs:323-337`) uses `extract_if` with the predicate `mutex_addr == addr && mutex.reference_count() <= 2`, meaning it only removes the mutex entry when the `Arc` strong count is ≤ 2 (i.e., only the BTreeMap and one clone hold references). The verified version completely omits reference-counting semantics, making it non-equivalent: in the original, `put_mutex` on a mutex with active references is a no-op (returns `Ok(())` without removal), while the verified version always removes.
  **Suggested Fix:** Model reference counts in the ghost state. Add a `ref_count: int` field to the ghost mutex map values (`Map<int, int>` where the int value represents the reference count). The `get_mutex` should increment the ref count on existing entries (modeling `clone()`), and `put_mutex` should only remove when the modeled ref count drops to the threshold (≤ 2). Alternatively, document this as a deliberate over-approximation in the trust assumptions, but this weakens the verification considerably.

- **Location:** `put_cond()` (exec: `process_state.rs:410-418`)
  **Description:** Same issue as `put_mutex`. The original (`mod.rs:382-396`) uses `extract_if` with `cond.reference_count() <= 1`, conditionally removing the condvar only when no other references exist. The verified version always removes. This means the verified code allows removing a condvar that still has active waiters, which the original prevents.
  **Suggested Fix:** Same approach as `put_mutex` — model the reference count in the ghost condvar map and only remove when the count drops to the threshold (≤ 1).

### High

- **Location:** `MUTEX_MAX()` / `COND_MAX()` (spec: `process_state.spec.rs:127-134`, exec: `process_state.rs:493-506`)
  **Description:** The verification hardcodes `MUTEX_MAX = 256` and `COND_MAX = 256`, but the actual kernel configuration (`build/kernel_config.toml:40,45`) sets `mutex_open_max = 32` and `cond_open_max = 32`. The trust assumption T1 in the module documentation states these should match, but the values are incorrect by a factor of 8×. This means the verification proves a weaker capacity bound than what the real kernel enforces.
  **Suggested Fix:** Change both constants to `32usize` to match the actual kernel configuration, or parameterize the spec constants and document the actual values used in deployment.

- **Location:** `remove_pmio()` (exec: `process_state.rs:456-490`)
  **Description:** The verified version uses `Seq::filter(|p: int| p != port_number@)` which removes **all** occurrences of the matching port number from the sequence. The original (`mod.rs:241-251`) uses `LinkedList::remove(index)` after `iter().position()`, which removes only the **first** occurrence. If duplicate port numbers ever exist in the list, the behaviors diverge. Additionally, the postcondition doesn't specify the resulting PMIO count (only that wf is preserved and old state had the port).
  **Suggested Fix:** Model the removal as removing only the element at the found index position rather than filtering all matches. Use `Seq::remove(idx)` where `idx` is a ghost index satisfying the position predicate. Add a postcondition relating the new PMIO count to the old count minus 1.

### Medium

- **Location:** `wf()` predicate (spec: `process_state.spec.rs:108-114`)
  **Description:** The well-formedness predicate does not include capacity bounds (`mutex_count <= MUTEX_MAX` and `cond_count <= COND_MAX`). While the exec code enforces these at runtime, the invariant doesn't capture them, meaning the verifier cannot prove that `mutex_count + 1` won't overflow `usize` in `get_mutex` without relying on the runtime check alone. The invariant is sufficient for current proofs because `MUTEX_MAX = 256 << usize::MAX`, but it misses an opportunity to prove the tighter property that capacity is always bounded.
  **Suggested Fix:** Add `self.mutex_count as nat <= Self::MUTEX_MAX() as nat` and `self.cond_count as nat <= Self::COND_MAX() as nat` to `wf()`. This strengthens the invariant and makes the capacity bound a proven property rather than an implicit consequence.

- **Location:** Coverage gap — 12 functions omitted
  **Description:** The following public functions from the original are not verified: `vmem()`, `vmem_mut()`, `copy_from_user_unaligned()`, `copy_to_user_unaligned()`, `add_event()`, `remove_event()`, `post_message()`, `receive_message()`, `add_mmio()`, `remove_mmio()`, `read_pmio()`, `write_pmio()`. While the documentation explains these interact with opaque boundary types (Vmem, EventOwnership, Mailbox, IoMemoryRegion, AnyIoPort), this leaves roughly half the module's public API unverified. The private helpers `get_pmio()` and `get_pmio_mut()` are also omitted.
  **Suggested Fix:** Even without modeling opaque types fully, frame-condition specifications (proving these functions don't modify mutex/cond/capability state) would add value. Consider adding stub specs with `external_body` that at least assert preservation of verified fields.

- **Location:** `get_mutex()` / `get_cond()` return type (exec: `process_state.rs:228`, `process_state.rs:334`)
  **Description:** The original `get_mutex` returns `Result<Mutex, Error>` (providing the mutex object to the caller), and `get_cond` returns `Result<Condvar, Error>`. The verified versions return `Result<(), Error>`, discarding the return value. This means the verification does not capture the contract that the returned object corresponds to the requested address and is a valid clone.
  **Suggested Fix:** Model the returned mutex/condvar as a ghost token or abstract handle to capture the correspondence between the address and the returned object.

- **Location:** `ProcessRefMut` / `ProcessRef` enums (original: `mod.rs:86-124`)
  **Description:** The `ProcessRefMut` and `ProcessRef` enums and their `state_mut()`/`state()` dispatch methods are not modeled. While these are accessor wrappers, they form part of the public API for the process state machine and mediate access to `ProcessState` from different process lifecycle states (Runnable, Running, Sleeping, Interrupted, Zombie).
  **Suggested Fix:** Add at minimum a specification that `state_mut()`/`state()` correctly dispatches to the inner `ProcessState` for each variant, or document this as out of scope.

### Low

- **Location:** `set_capability()` / `clear_capability()` postconditions (exec: `process_state.rs:170,191`)
  **Description:** The postconditions include `old(self).capabilities.wf() ==> self.wf()` which is unnecessarily weak. Since the precondition requires `old(self).wf()`, which implies `old(self).capabilities.wf()`, the conditional always holds and the postcondition could simply be `self.wf()`. The current form may confuse readers into thinking wf preservation is conditional.
  **Suggested Fix:** Simplify to `self.wf()` in the ensures clause, since `old(self).wf()` is already required.

- **Location:** `get_mutex()` / `get_cond()` API shape (exec: `process_state.rs:228,334`)
  **Description:** The verified functions take `already_present: bool` as an explicit runtime parameter tied to ghost state via a precondition. While this is a valid Verus pattern for abstracting `BTreeMap::contains_key`, it shifts complexity to the caller who must maintain and pass this oracle. The original API is self-contained (`entry().or_insert_with()`).
  **Suggested Fix:** This is an acceptable verification pattern. Document it clearly at the call site level so maintainers understand the oracle pattern. No code change needed.

- **Location:** `Debug` trait impl (original: `mod.rs:411-415`)
  **Description:** The `Debug` implementation for `ProcessState` is not modeled. This is purely a display concern and has no functional impact.
  **Suggested Fix:** No action needed. Correctly omitted.

## Positive Observations

- **Verification passes cleanly:** 33 verified, 0 errors. No `assume`, `external_body`, or `trusted` annotations in the core module — the verification is fully mechanized for the modeled subset.
- **Well-structured separation:** The spec/proof/exec split is clean and follows good Verus conventions. Specs define abstract state accessors, proofs provide reusable lemmas, and exec code contains only implementation with embedded pre/postconditions.
- **Thorough documentation:** The module-level documentation clearly states the verification model, trust assumptions, and scope boundaries. This is exemplary for a verified module.
- **Frame conditions:** All mutating functions specify comprehensive frame conditions (which fields are preserved), preventing unintended state corruption.
- **PID immutability:** Proven across all operations via both postconditions and dedicated lemmas.
- **Ghost state design:** Using `Ghost<Map<int, int>>` and `Ghost<Seq<int>>` to model BTreeMap and LinkedList is a reasonable abstraction that enables property verification without modeling complex container internals.
- **Proof lemmas are well-organized:** The proof file provides lemmas for construction, PID immutability, map operations, and well-formedness preservation in clearly delineated sections.

## Summary

The verification demonstrates competent Verus technique with clean separation, thorough frame conditions, and no unsound escape hatches. However, it has two **critical semantic equivalence issues**: `put_mutex` and `put_cond` omit reference-counting logic that is central to the original's correctness (safe cleanup only when no other references exist). This means the verified model permits mutex/condvar removal that the original prevents, fundamentally misrepresenting the resource lifecycle protocol.

Additionally, the capacity constants are wrong (256 vs. actual 32), `remove_pmio` has a filter-vs-remove-first semantic gap, and roughly half the public API is unverified. The `wf()` invariant could be strengthened to include capacity bounds.

**Recommendations (priority order):**
1. Model reference counts in `put_mutex`/`put_cond` to match the conditional removal semantics.
2. Fix MUTEX_MAX/COND_MAX to match the actual kernel configuration (32).
3. Fix `remove_pmio` to remove only the first matching element.
4. Strengthen `wf()` with capacity bounds.
5. Add frame-condition stubs for omitted functions to increase coverage.
