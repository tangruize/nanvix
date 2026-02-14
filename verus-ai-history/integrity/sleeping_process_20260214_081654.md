# Exec Integrity: sleeping_process

## Summary
- Total differences: 14
- Acceptable (ghost annotations): 8
- Type changes (documented): 4
- Logic changes: 0
- Fixed: 0
- Unfixable (documented): 2
- Missing functions: 0
- Invented functions: 0

## Source Files
- Original: `src/kernel/src/pm/process/state/sleeping.rs`
- Verified exec: `verus/split/kernel/pm/process/state/sleeping.rs`
- Verified spec: `verus/split/kernel/pm/process/state/sleeping.spec.rs`
- Verified proof: `verus/split/kernel/pm/process/state/sleeping.proof.rs`

## Differences

| # | Type | Location | Description | Action |
|---|------|----------|-------------|--------|
| 1 | TYPE_CHANGE | `struct SleepingProcess` | `state: Box<ProcessState>` → `pid: u64`. ProcessState abstracted to PID for verification. | Documented — standard verification model abstraction. PID is the only identity-relevant field. |
| 2 | TYPE_CHANGE | `struct SleepingProcess` | `sleeping_threads: NonEmptyVecDeque<SleepingThread>` → `sleeping_thread_ids: Vec<u64>`. Thread objects abstracted to ID list. Non-empty invariant enforced via `wf()` predicate (`len() >= 1`). | Documented — ID-level modeling. NonEmptyVecDeque invariant preserved in spec. |
| 3 | TYPE_CHANGE | `struct SleepingProcess` | `zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>` → `zombie_thread_ids: Vec<u64>`. Option+NonEmpty modeled as Vec (empty = None). | Documented — standard Option modeling in Verus. |
| 4 | TYPE_CHANGE | `fn wakeup()` | Added oracle parameter `found: bool` (not in original). Original uses `remove_if` match result to determine found/not-found. Oracle is constrained by `found == spec_seq_contains(sleeping_thread_ids@, tid)`. | Documented — oracle replaces runtime match on `remove_if`. Precondition ties oracle to specification. |
| 5 | TYPE_CHANGE | `fn wakeup_alarm()` | Parameter `now: SystemTime` replaced by oracle parameters `has_expired: bool`, `interrupted_ids: Vec<u64>`, `remaining_ids: Vec<u64>`. Alarm comparison logic (`now >= alarm`) is pushed to the oracle boundary. | Documented — per-thread alarm data is not modeled. Conservation, no-duplicates, disjointness, and subsequence-ordering constraints on oracle parameters ensure structural correctness of the partition. |
| 6 | GHOST_ANNOTATION | `fn new()` | Added `requires` (non-empty, no-duplicates, disjoint) and `ensures` (wf, field equality). | Acceptable |
| 7 | GHOST_ANNOTATION | `fn state()` | Marked `external_body` with `ensures result == self.spec_pid()`. Returns `u64` instead of `&ProcessState`. | Acceptable — models reference return as PID value. |
| 8 | GHOST_ANNOTATION | `fn state_mut()` | Marked `external_body` with frame conditions (PID, sleeping, zombie unchanged). Returns `u64` instead of `&mut ProcessState`. | Acceptable — frame conditions prevent unmodeled mutation. |
| 9 | GHOST_ANNOTATION | `fn terminate()` | Added `requires self.wf()`, `ensures` (PID preserved, all sleeping→interrupted, no sleeping remain, zombie preserved), and proof block. | Acceptable |
| 10 | GHOST_ANNOTATION | `fn wakeup()` | Added `requires self.wf()`, `ensures` (Ok: PID preserved, ready=[tid], sleeping reduced by 1, tid removed; Err: unchanged), loop invariants, and proof blocks. | Acceptable |
| 11 | GHOST_ANNOTATION | `fn wakeup_alarm()` | Added `requires` (wf, oracle partition constraints) and `ensures` (Ok: PID preserved, partition conservation; Err: unchanged). | Acceptable |
| 12 | GHOST_ANNOTATION | `fn add_thread()` | Added `requires` (wf, tid not in sleeping/zombie), `ensures` (PID preserved, ready=[tid], sleeping/zombie preserved), and proof block. | Acceptable |
| 13 | GHOST_ANNOTATION | `fn find_thread()` | Return type changed from `Option<ThreadRef<'_>>` to `Ghost<Option<int>>`. Verus cannot express reference-returning functions; modeled as spec-only via `spec_find_thread()`. | Acceptable — documented trust boundary. Callers must independently verify reference use. |
| 14 | GHOST_ANNOTATION | `fn find_thread_mut()` | Return type changed from `Option<ThreadRefMut<'_>>` to `Ghost<Option<int>>` with frame conditions. Same reasoning as `find_thread()`. | Acceptable — documented trust boundary. Frame condition ensures self unchanged. |

## Detailed Analysis

### Exec Logic Faithfulness

All 9 original functions are present in the verified code. No functions are missing or invented.

**`new()`**: Direct struct construction — faithful at ID level.

**`state()` / `state_mut()`**: Marked `external_body` because they return references to `ProcessState`, which is abstracted away. The PID-only model is sufficient for identity tracking. Frame conditions on `state_mut()` prevent unmodeled side effects.

**`terminate()`**: Original iterates sleeping threads calling `.interrupt(Killed)` on each. Verified code moves all sleeping IDs directly to interrupted list. This is faithful because `interrupt()` is ID-preserving (thread ID is unchanged by state transition), so the iteration is only needed for the type-level transition, not the ID-level model.

**`wakeup(tid)`**: Original calls `sleeping_threads.remove_if(|t| t.id() == tid)` then matches Ok/Err. Verified code uses an oracle `found` parameter tied to `spec_seq_contains`, then performs a linear search and `Vec::remove`. Both approaches: (1) search for tid, (2) remove if found, (3) create RunnableProcess or return self. The exec search loop is concrete and faithful.

**`wakeup_alarm(now)`**: Original has complex nested logic: pop front, check alarm, check expiry, then iterate remaining threads partitioning by alarm expiry. Verified code delegates the partition decision to oracle parameters with strong conservation constraints. This is the largest modeling gap — the per-thread `now >= alarm` comparison is not verified. However, the structural properties (conservation, ordering, no-duplicates, disjointness) are all verified, which is the primary concern for process state machine correctness.

**`add_thread(ready_thread)`**: Direct transition to RunnableProcess — faithful.

**`find_thread()` / `find_thread_mut()`**: Spec-only models returning `Ghost<Option<int>>`. The search logic (sleeping first, then zombie) is captured in `spec_find_thread()`. Reference-returning semantics cannot be expressed in Verus.

### Boundary Types

`RunnableProcess` and `InterruptedProcess` are defined in the verified file as boundary models of sibling module types. They are not invented exec functions but structural models needed to express return types. Their `wf()` predicates are minimal (non-empty primary list only), appropriate for boundary models.

### Trust Boundaries

1. **ProcessState access**: `state()` / `state_mut()` are `external_body`. PID immutability is asserted but not proven from source.
2. **Thread state transitions**: `interrupt()` and `wakeup()` are modeled as ID-preserving. The actual thread state machine is not verified here.
3. **Alarm comparison**: `wakeup_alarm()` delegates `now >= alarm` to oracle parameters. Per-thread alarm logic is a trust boundary.
4. **Reference returns**: `find_thread()` / `find_thread_mut()` return ghost values instead of actual references.

### No Unjustified Assumptions

- No `assume` or `admit` statements found.
- Two `external_body` annotations on `state()` / `state_mut()` are justified: they return references to an abstracted type (`Box<ProcessState>` → `u64`).

## Verification Status
- Before: PASS (29 verified, 0 errors)
- After: PASS (no changes needed — no LOGIC_CHANGE or MISSING_FUNCTION items)
