# Exec Consistency Fix: kredzone

## Summary
- Mismatches fixed: 0 (both are documented semantic equivalences)
- Missing functions added: 0
- Documented equivalences: 2 (store, load)

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `store` [store.diff](store.diff) [store_source.rs](store_source.rs) [store_verus.rs](store_verus.rs) | Documented equivalence | Three documented divergences: (1) `KREDZONE_SIZE / mem::size_of::<usize>()` replaced by `NUM_ENTRIES` — semantically equivalent, proven by `lemma_entry_size_matches_target`; (2) `error!()` logging omitted — side-effect irrelevant to verification, documented in module header; (3) inline `unsafe { ptr.write_volatile() }` extracted to `raw_store()` — structural change to minimize `external_body` scope. |
| `load` [load.diff](load.diff) [load_source.rs](load_source.rs) [load_verus.rs](load_verus.rs) | Documented equivalence | Same three divergence patterns as `store`: (1) bounds check uses equivalent `NUM_ENTRIES`; (2) `error!()` logging omitted; (3) inline `unsafe { ptr.read_volatile() }` extracted to `raw_load()`. |
| `raw_store` [raw_store_verus.rs](raw_store_verus.rs) | Justified extra | Extracted `external_body` helper isolating the trusted volatile write. Required to keep the bounds check verified while only marking the volatile op as trusted. |
| `raw_load` [raw_load_verus.rs](raw_load_verus.rs) | Justified extra | Extracted `external_body` helper isolating the trusted volatile read. Same rationale as `raw_store`. |
| `store_with_ghost` [store_with_ghost_verus.rs](store_with_ghost_verus.rs) | Justified extra | Verified wrapper threading ghost state through `store()` for algebraic reasoning (read-after-write, commutativity). Does not alter original API. |
| `load_with_ghost` [load_with_ghost_verus.rs](load_with_ghost_verus.rs) | Justified extra | Verified wrapper using ghost state to relate return value to abstract model. Does not alter original API. |
| `init_kredzone` [init_kredzone_verus.rs](init_kredzone_verus.rs) | Justified extra | Explicit zero-initialization of all kredzone entries, providing an alternative to trust assumption T4 (BSS zero-init). Calls verified `store()` in a loop. |

## Detailed Equivalence Analysis

### `store` — Bounds Check Equivalence

Original: `index >= KREDZONE_SIZE / mem::size_of::<usize>()`
Verus:    `index >= NUM_ENTRIES`

Where `NUM_ENTRIES = KREDZONE_SIZE / ENTRY_SIZE` and `ENTRY_SIZE` is conditionally compiled:
- `target_pointer_width = "32"`: `ENTRY_SIZE = 4 = size_of::<usize>()`
- `target_pointer_width = "64"`: `ENTRY_SIZE = 8 = size_of::<usize>()`

This equivalence is verified by `lemma_entry_size_matches_target()` in the proof file.
Verus cannot directly reason about `mem::size_of`, so a compile-time literal is used instead.

### `store` / `load` — Logging Omission

The `error!()` macro calls are intentionally omitted in the verified version to avoid
side effects during verification. This is documented in the module header under
"Divergences from Original Implementation". Logging does not affect control flow or
return values.

### `store` / `load` — Unsafe Block Extraction

The original inline `unsafe` blocks are replaced by calls to `raw_store()`/`raw_load()`,
both marked `#[verifier::external_body]`. This is a standard Verus pattern to minimize
the trusted code surface: the bounds check remains verified, and only the volatile
memory operation is trusted. The exec behavior is identical.

## Verification: PASS
- 36 verified, 0 errors
- Command: `./verus-ai/scripts/verify.sh kredzone`
