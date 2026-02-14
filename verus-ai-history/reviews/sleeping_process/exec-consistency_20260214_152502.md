# Review: sleeping_process Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### Minor

1. **`wakeup_alarm()` oracle model is a significant abstraction gap.**
   The original `wakeup_alarm()` (lines 107–176) contains non-trivial control flow: it pops the front thread, checks its alarm, then iterates remaining threads partitioning by alarm expiry. The Verus model replaces this entire loop with oracle parameters (`has_expired`, `interrupted_ids`, `remaining_ids`). While the oracle preconditions enforce conservation (length, content containment, no-duplicates, disjointness, subsequence ordering), the actual per-thread alarm comparison logic (`now >= alarm`) is fully elided. This is a deliberate and documented trust boundary, but it means the partition correctness itself is not verified — only that *if* the partition is correct, the state transition is correct. The subsequence constraints (lines 486–489) do capture the stable partition ordering of the original, which is good.

2. **`state()` / `state_mut()` external_body with `unimplemented!()` body.**
   Both functions (lines 200–227) use `#[verifier::external_body]` with `unimplemented!()` as the body. This is standard Verus practice for functions whose types cannot be expressed, and both are well-documented with TRUST comments and frame conditions. The `state_mut()` frame condition correctly ensures PID immutability and list preservation (`self@.sleeping_thread_ids =~= old(self)@.sleeping_thread_ids`, `self@.zombie_thread_ids =~= old(self)@.zombie_thread_ids`). Sound.

3. **`new()` visibility change: `pub(super)` → `pub`.**
   The original has `pub(super) fn new(...)` (line 51), while the Verus version uses `pub fn new(...)` (line 140). This is a minor visibility widening that doesn't affect correctness but deviates from the original's encapsulation. Not documented in the consistency report.

4. **`add_thread()` drops `mut self` → `self`.**
   The original takes `mut self` (line 178) while the Verus version takes `self` (line 561). The `mut` in the original is only needed for `self.zombie_threads.take()` which has no analog in the simplified model. This is semantically correct — both consume `self`.

5. **`add_thread()` drops `trace!()` logging.**
   The original (line 179) has `trace!("self.pid={:?}, ready_thread={:?}", self.state.pid, ready_thread)`. The Verus model omits this. Acceptable — logging has no semantic effect.

### Observations

- **All 9 original functions are present in the Verus exec code** (`new`, `state`, `state_mut`, `terminate`, `wakeup`, `wakeup_alarm`, `add_thread`, `find_thread`, `find_thread_mut`). No functions are missing.

- **Struct field mapping is sound.** `Box<ProcessState>` → `pid: u64`, `NonEmptyVecDeque<SleepingThread>` → `sleeping_thread_ids: Vec<u64>`, `Option<NonEmptyVecDeque<ZombieThread>>` → `zombie_thread_ids: Vec<u64>`. The model correctly captures that an empty `Vec<u64>` represents `None` and a non-empty one represents `Some(...)`.

- **Boundary types (`RunnableProcess`, `InterruptedProcess`) are well-scoped.** They model just enough of sibling module types to serve as return types with the correct field structure.

- **`wakeup()` algorithm is faithfully modeled.** The original uses `NonEmptyVecDeque::remove_if(|t| t.id() == tid)`. The Verus version uses an oracle `found` parameter (tied to `spec_seq_contains`) plus a concrete linear scan loop (lines 356–370) with proper invariants, followed by `Vec::remove`. The search logic, removal, and error path are semantically equivalent. The proof that the search must succeed (lines 372–381) is clean.

- **`terminate()` simplification is correct.** The original iterates through sleeping threads calling `interrupt(InterruptReason::Killed)` on each. Since `interrupt()` is ID-preserving (documented trust boundary), the Verus model correctly moves the sleeping ID list directly to the interrupted ID list.

- **Well-formedness (`wf()`) is comprehensive.** It enforces: (1) at least one sleeping thread, (2) no duplicate IDs within sleeping list, (3) no duplicate IDs within zombie list, (4) sleeping and zombie lists are disjoint. All operations prove `wf()` preservation.

- **Proof infrastructure is solid.** 30 verified obligations, 0 errors. The bridging lemmas (`lemma_find_thread_view_equiv`, `lemma_view_wf_equiv`, various `_refines_spec` lemmas) correctly connect exec-level and view-level reasoning. The view-level abstract transition functions enable downstream compositional reasoning.

- **No `assume` or `admit` used anywhere.** Only 2 `external_body` annotations, both justified and documented.

## Verification Result

- **30 verified, 0 errors** (confirmed by running `./verus-ai/scripts/verify.sh sleeping_process`).
- 2 `external_body` functions (`state()`, `state_mut()`) — both justified as Verus reference-type limitations.
- 0 `assume`, 0 `admit`.

## Summary

The exec consistency fix is well-executed. All 9 original functions are present with semantically equivalent implementations. The type abstractions (complex kernel types → `u64` IDs and `Vec<u64>` lists) are appropriate for verification modeling and are thoroughly documented in the module header, consistency report, and inline TRUST comments. The most significant abstraction is in `wakeup_alarm()` where alarm-based partitioning is replaced by oracle parameters — this is a genuine trust boundary but is defended by strong conservation and ordering preconditions. The proof infrastructure is comprehensive with view-level bridging lemmas enabling downstream composition. Verification passes cleanly with 30 obligations verified and no cheating patterns beyond the two justified `external_body` annotations. The only undocumented deviation is the `pub(super)` → `pub` visibility change on `new()`, which is cosmetic.
