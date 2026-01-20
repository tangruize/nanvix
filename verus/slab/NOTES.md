# AI-Assisted Formal Verification of Slab Allocator

---

## What I Did

- Used AI to formally verify the **slab allocator** in Nanvix
- Focus: **memory management safety**
- Result: **~3.7K lines** of verification code, **86 verified items**

---

## Setup

| Role | Model |
|------|-------|
| **Prover** | Claude Opus 4.5 |
| **Reviewers** | Claude Opus 4.5, GPT 5.1 Codex Max, Gemini 3 Pro |

- Opus performed best for Verus
- Multiple reviewers for cross-validation

---

## Workflow

```
1. UNDERSTAND  →  2. ABSTRACT  →  3. PROVE  →  4. REVIEW  →  5. ITERATE  →  6. VALIDATE
```

| Phase | What Happens |
|-------|--------------|
| **1. Understand** | AI reads slab code, identifies specs/invariants/properties |
| **2. Abstract** | Write specs, views, axioms for bitmap/raw_array |
| **3. Prove** | Write proofs, run Verus, fix errors, repeat |
| **4. Review** | Other AIs review for weak/missing/incorrect specs |
| **5. Iterate** | Fix reviewer comments until no objections |
| **6. Validate** | Inject bugs, check they're caught |

---

## Cost & Effort

| Metric | Value |
|--------|-------|
| Copilot premium requests | ~200 |
| Opus rate | 3x per prompt |
| API time | ~3 hours |
| Human time | ~12 hours |

> Note: 1000 request limit is soft, not hard cutoff

---

## Results

| Metric | Value |
|--------|-------|
| Verified items | **86** |
| Errors | **0** |
| Code expansion | **15x** (224 → 3,263 lines) |
| Assumes in slab_core.rs | **0** |
| external_body on slab functions | **0** |

---

## What Was Verified

**Functions:**
- `from_raw_parts` - creates valid slab
- `new` - creates slab from components  
- `allocate` - returns valid address, marks block
- `deallocate` - bounds checks, marks block free

**Properties (15+ lemmas):**
- Memory bounds safety
- Block disjointness
- Metadata/data disjointness
- Address-index inverses
- Liveness guarantees
- Allocation uniqueness
- Memory conservation

---

## Invariant (11 conditions)

1. `block_size > 0`
2. `num_data_blocks > 0`
3. `num_index_blocks > 0`
4. Block counts match bitmap
5. Index blocks always allocated
6. `data_addr > 0`
7. `num_data_blocks * block_size <= usize::MAX`
8. `data_addr + num_data_blocks * block_size <= usize::MAX`
9. Metadata/data disjoint
10. Power-of-two block size
11. Data address aligned

---

## Bug Injection Test

| Bug Injected | Caught? |
|--------------|---------|
| Off-by-one address (+1) | ✅ |
| Forgot num_index_blocks offset | ✅ |
| Removed power-of-two invariant | ✅ |
| Wrong block in frame condition | ✅ |

All 4 bugs caught, including subtle frame condition error.

---

## File Structure

```
verus/slab/
├── lib.rs            # Entry point (26 lines)
├── error.rs          # Error types - trusted (73 lines)
├── raw_array.rs      # RawArray - trusted (144 lines)
├── bitmap.rs         # Bitmap - trusted (209 lines)
├── slab_core.rs      # FULLY VERIFIED (3,263 lines)
└── *.md              # Documentation (~1,600 lines)
```

---

## Observations: Opus

**Strengths:**
- Best at Verus syntax and proof strategies
- High throughput

**Challenges:**
- Context window exhausts easily → truncated outputs
- Makes very large edits at once
- Hard to track what changed

---

## Observations: GPT 5.1 Codex Max

**Issues:**
- Crashes/timeouts during long sessions
- Trial-and-error: many edits → many errors → slow fixes
- Introduces many `external_body` to pass verification
- Modifies original code too much
- Changes preconditions to assertions (not sound!)

---

## Issues Found During Verification

| Issue | Status |
|-------|--------|
| Math calculations axiomatized | Acceptable, documented |
| Overflow checks assumed | Fixed with bounds invariants |
| Original code modified | Need to check compatibility |
| Error cases not verified | Just "precondition not satisfied" |
| 10x code expansion | Industry normal |

---

## Resolution Process

1. Multiple AIs review final state
2. Each gives feedback
3. Opus fixes
4. Repeat until no objections
5. Gemini & GPT took turns

**Final state:**
- 0 assumes
- 0 external_body on slab functions
- Only trusted: Bitmap, RawArray

---

## Future Improvements

| Area | Need |
|------|------|
| **Logging** | Git-based prompt/output logging |
| **Tooling** | Verus wrapper with auto-commit, cheating detection |
| **Workflow** | Semi-automatic with checkpoints |
| **Prompts** | More explicit constraints, spec quality metrics |
| **Model strategies** | Constrain each model's behavior |

---

## Key Learnings

| Learning | Detail |
|----------|--------|
| Multi-AI review works | Different models catch different issues |
| 15x expansion is normal | seL4: 20:1, CompCert: 10:1 |
| Document trusted boundaries | Axioms must be correct for soundness |
| Iterative refinement works | external_body → real proofs gradually |
| Verified code is compatible | Additive changes, simple casts at API |

---

## Discussion Questions

1. Is 15x expansion acceptable for slab allocator?
2. How to integrate with production code?
3. What else should we verify?
4. Can we automate multi-AI review?
5. What tooling would help?

---

## Summary

- AI-assisted verification **works** for systems code
- Multi-AI review improves quality
- Tooling and workflow need improvement
- Complete codebase in attached zip

---

*2026-01-13*
